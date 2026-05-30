use myzero2prod::configuration::get_configuration;
use std::io;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use myzero2prod::startup::run;
use myzero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let subscriber = get_subscriber("zero2prod", "info", io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read config");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
