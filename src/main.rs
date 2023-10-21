#![warn(clippy::pedantic)]

use zero_to_prod::telemetry;
use zero_to_prod::{configuration::get_configuration, startup::Application};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber =
        telemetry::get_subscriber("zero-to-prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;

    Ok(())
}
