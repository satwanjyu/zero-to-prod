use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::utils::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber."
    skip(pool, parameters),
)]
pub async fn confirm(
    pool: web::Data<PgPool>,
    parameters: web::Query<Parameters>,
) -> Result<HttpResponse, ConfirmationError> {
    let subscriber_id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id from token")?
        .ok_or(ConfirmationError::UnknownToken)?;
    confirm_subscriber(&pool, subscriber_id)
        .await
        .context("Failed to update subscriber status to `confirmed`")?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("There is no subscriber associated with the provided token.")]
    UnknownToken,
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmationError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmationError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ConfirmationError::UnknownToken => StatusCode::BAD_REQUEST,
        }
    }
}

#[tracing::instrument(
    "Get subscriber_id from subscription_token.",
    skip(pool, subscription_token)
)]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let subscriber_id = match sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    {
        Ok(record) => record.map(|r| r.subscriber_id),
        Err(e) => {
            return Err(e);
        }
    };

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Change subscriber status to confirmed",
    skip(pool, subscriber_id)
)]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    match sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };

    Ok(())
}
