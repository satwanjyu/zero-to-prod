#![warn(clippy::pedantic)]

use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero_to_prod::{
    configuration::get_configuration,
    startup,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero-to-prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let listener = TcpListener::bind(format!(
        "{}:{}",
        &configuration.application.host, &configuration.application.port
    ))
    .expect("Failed to bind to address");

    startup::run(listener, connection_pool)?.await
}
