use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: Client,
    api_user: String, // <-- 保存 api_user
    api_key: SecretString,
}

#[derive(serde::Serialize)] // 👈 核心：允许这个结构体被序列化
struct SendEmailRequest<'a> {
    #[serde(rename = "apiUser")] // 👈 自动映射成 SendCloud 的参数名
    api_user: &'a str,

    #[serde(rename = "apiKey")]
    api_key: &'a str,

    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html: &'a str, // SendCloud 接收 HTML 正文的参数名叫 html
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        api_user: String,
        api_key: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
            api_user,
            base_url,
            sender,
            api_key,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        _text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/mail/send", self.base_url);
        let request_body = SendEmailRequest {
            api_user: self.api_user.as_str(),
            api_key: self.api_key.expose_secret(),
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html: html_content,
        };

        self.http_client
            .post(&url)
            .form(&request_body) // <-- 关键点：用 .form() 提交 x-www-form-urlencoded 数据
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use claim::{assert_err, assert_ok};
    use fake::faker::lorem::en::Paragraph;
    use fake::{
        Fake,
        faker::{internet::en::SafeEmail, lorem::en::Sentence},
    };
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a ranadom subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            "fake_user".to_string(), // 随便传个假的 api_user 喂给测试
            "fake_key".to_string().into(),
            std::time::Duration::from_millis(200),
        )
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            // 1. 关键改动：将表单的字节流（bytes）解析为 HashMap
            let result: Result<std::collections::HashMap<String, String>, _> =
                serde_urlencoded::from_bytes(&request.body);

            if let Ok(body) = result {
                // 2. 关键改动：检查 SendCloud 要求的表单 key 是否全都存在
                body.contains_key("apiUser")
                    && body.contains_key("apiKey")
                    && body.contains_key("from")
                    && body.contains_key("to")
                    && body.contains_key("subject")
                    && body.contains_key("html")
            } else {
                // 如果表单解析失败，说明请求格式不对，返回 false
                false
            }
        }
    }
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = email();
        let subject = subject();
        let content = content();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let subscriber_email = email();
        let subject = subject();
        let content = content();

        // 我们不会复制其他测试中的所有匹配器。
        // 这个测试的目的不是断言我们的请求
        // 我们添加触发我们想要的路径所需的最低限度
        // 在 `send_email` 中进行测试。
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_of_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let subscriber_email = email();
        let subject = subject();
        let content = content();

        Mock::given(any())
            // Not a 200 anymore!
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(outcome);
    }
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let subscriber_email = email();
        let subject = subject();
        let content = content();

        let response = ResponseTemplate::new(200)
            // 3 minutes!
            .set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(outcome);
    }
}
