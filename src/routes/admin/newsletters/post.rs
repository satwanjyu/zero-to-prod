use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

use crate::utils::e500;
use crate::{domain::SubscriberEmail, email_client::EmailClient};

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    html_content: String,
    text_content: String,
}

#[tracing::instrument(name = "Publish a newsletter issue", skip(form, pool, email_client))]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &form.title,
                        &form.html_content,
                        &form.text_content,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(e) => {
                tracing::warn!(
                    error.cause_chain = ?e,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid.",
                );
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers = sqlx::query!(
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| match SubscriberEmail::parse(r.email) {
        Ok(email) => Ok(ConfirmedSubscriber { email }),
        Err(e) => Err(anyhow::anyhow!(e)),
    })
    .collect();

    Ok(confirmed_subscribers)
}
