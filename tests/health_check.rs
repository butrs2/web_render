use secrecy::ExposeSecret;
use std::net::TcpListener;
use std::sync::Once;

//！测试/health_check.rs
// `tokio::test` 相当于 `tokio::main` 的测试。
// 它还使您不必指定`#[test]`属性。
//
// 您可以使用以下命令检查生成的代码
// `cargo Expand --test health_check` (<- 测试文件的名称)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let TestApp { port, db_pool } = spawn_app().await;
    // 我们需要引入 `reqwest`
    // 对我们的应用程序执行 HTTP 请求。
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("http://127.0.0.1:{}/health_check", port))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

use myzero2prod::configuration::{DatabaseSettings, get_configuration};
use myzero2prod::startup::run;
use myzero2prod::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let TestApp { port, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();
    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("http://127.0.0.1:{}/subscriptions", &port))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // 1. 把临时的连接字符串提取成独立变量
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(sqlx::AssertSqlSafe(format!(
            r#"CREATE DATABASE "{}";"#,
            config.database_name
        )))
        .await
        .expect("Failed to create database.");

    // 3. 把第二个临时的连接字符串也提取出来
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    // Migrate database
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let TestApp { port, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("http://127.0.0.1:{}/subscriptions", &port))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // 测试失败时附加自定义错误消息
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    // Arrange
    let TestApp { port, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("http://127.0.0.1:{}/subscriptions", &port))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}

pub struct TestApp {
    pub port: u16,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    static TRACING: Once = Once::new();
    TRACING.call_once(|| {
        let default_filter_level = "info";
        let subscriber_name = "test";
        //      我们不能根据“TEST_LOG”的值将“get_subscriber”的输出赋值给变量
        //      由于汇是“get_subscriber”返回类型的一部分，因此它们不是
        //      同一种类型。我们可以绕过这个问题，但这是最直接的前进方式
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut configuration = get_configuration().expect("");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let port = listener.local_addr().unwrap().port();
    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        port,
        db_pool: connection_pool,
    }
}
