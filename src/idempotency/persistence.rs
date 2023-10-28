use crate::authentication::UserId;
use crate::idempotency::IdempotencyKey;
use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use sqlx::postgres::{PgHasArrayType, PgTypeInfo};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_header_pair")
    }
}

async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: &UserId,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT
            response_status_code as "response_status_code!",
            response_headers as "response_headers!: Vec<HeaderPairRecord>",
            response_body as "response_body!"
        FROM idempotency
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id.0,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    let response = if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut builder = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in r.response_headers {
            builder.append_header((name, value));
        }
        let response = builder.body(r.response_body);

        Some(response)
    } else {
        None
    };

    Ok(response)
}

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
    idempotency_key: &IdempotencyKey,
    user_id: &UserId,
    http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    let (head, body) = http_response.into_parts();
    let status_code = head.status().as_u16() as i16;
    let headers: Vec<HeaderPairRecord> = head
        .headers()
        .iter()
        .map(|(name, value)| HeaderPairRecord {
            name: name.as_str().to_owned(),
            value: value.as_bytes().to_owned(),
        })
        .collect();
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    sqlx::query_unchecked!(
        r#"
        UPDATE idempotency
        SET
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id.0,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body.as_ref()
    )
    .execute(transaction.as_mut())
    .await?;
    transaction.commit().await?;

    let http_response = head.set_body(body).map_into_boxed_body();

    Ok(http_response)
}

pub enum NextAction {
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
}

pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: &UserId,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let n_inserted_rows = sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            created_at
        )
        VALUES ($1, $2, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id.0,
        idempotency_key.as_ref()
    )
    .execute(transaction.as_mut())
    .await?
    .rows_affected();
    let next_action = if n_inserted_rows > 0 {
        NextAction::StartProcessing(transaction)
    } else {
        let saved_response = get_saved_response(pool, idempotency_key, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("We expected a saved response, we didn't find it"))?;
        NextAction::ReturnSavedResponse(saved_response)
    };

    Ok(next_action)
}
