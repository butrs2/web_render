use myzero2prod::configuration::get_configuration;
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use secrecy::ExposeSecret;
use myzero2prod::startup::run;
use myzero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let subscriber = get_subscriber("zero2prod", "info", io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read config");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);

    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
