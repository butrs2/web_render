use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        // 2. 直接调用 validator 库提供的标准邮箱验证函数
        if ValidateEmail::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

impl AsRef<String> for SubscriberEmail {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use fake::locales;
    use fake::locales::Data;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    // 2. 为它实现 quickcheck 的 Arbitrary 特征
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // 直接使用 quickcheck 自己的 choose 方法，不跨库传递 g
            let username = g
                .choose(locales::EN::NAME_FIRST_NAME)
                .unwrap()
                .to_lowercase();

            let domain = g.choose(&["com", "net", "org"]).unwrap();
            let email = format!("{username}@example.{domain}");

            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        dbg!(&valid_email);
        SubscriberEmail::parse(valid_email.0).is_ok()
    }
}
