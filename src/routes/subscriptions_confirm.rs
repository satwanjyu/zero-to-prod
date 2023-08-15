use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber."
    skip(pool, parameters)
)]
pub async fn confirm(pool: web::Data<PgPool>, parameters: web::Query<Parameters>) -> HttpResponse {
    let subscriber_id =
        match get_subscriber_id_from_token(&pool, &parameters.subscription_token).await {
            Ok(id) => id,
            Err(_) => {
                tracing::error!("Failed to get subscriber_id from subscription_token");
                return HttpResponse::InternalServerError().finish();
            }
        };
    let subscriber_id = match subscriber_id {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };
    match confirm_subscriber(&pool, subscriber_id).await {
        Ok(_) => {}
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().finish()
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
            tracing::error!("Failed to execute query: {:?}", e);
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
            tracing::error!("Failed to execute query: {:?}", e);
            return Err(e);
        }
    };

    Ok(())
}
