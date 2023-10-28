#![warn(clippy::pedantic)]

use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use zero_to_prod::issue_delivery_worker::run_worker_until_stopped;
use zero_to_prod::telemetry;
use zero_to_prod::{configuration::get_configuration, startup::Application};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber =
        telemetry::get_subscriber("zero-to-prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = worker_task => report_exit("Background worker", o),
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::error!("{task_name} has exited");
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error_message = %e,
                "{task_name} has failed"
            );
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{task_name} task faild to complete"
            );
        }
    }
}
