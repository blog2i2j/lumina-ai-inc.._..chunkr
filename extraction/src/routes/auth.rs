use crate::models::api::api_key::{
    AccessLevel, ApiKey, ApiKeyLimit, ApiKeyUsage, ApiRequest, UsageType,
};
use crate::utils::db::deadpool_postgres::{Client, Pool};
use actix_web::{web, Error, HttpMessage, HttpRequest, HttpResponse};
use prefixed_api_key::PrefixedApiKeyController;
use sha2::Sha256;
use uuid::Uuid;

use chrono::{DateTime, Utc};
use rand::rngs::OsRng;

pub async fn put_api_key(
    req: HttpRequest,
    request_payload: web::Json<ApiRequest>,
) -> Result<HttpResponse, Error> {
    let request = request_payload.into_inner();
    let pool = req.app_data::<web::Data<Pool>>().unwrap();
    let new_key = create_api_key(request, &pool).await?;
    Ok(HttpResponse::Ok().json(new_key))
}

pub async fn create_api_key(
    request: ApiRequest,
    pool: &Pool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut pg_client: Client = pool.get().await?;

    let controller: PrefixedApiKeyController<OsRng, Sha256> = PrefixedApiKeyController::configure()
        .prefix("lu".to_owned())
        .seam_defaults()
        .finalize()?;

    let (pak, _hash) = controller.try_generate_key_and_hash()?;
    let key = pak.to_string();

    let api_key = ApiKey {
        key: key.clone(),
        user_id: request.user_id,
        dataset_id: None,
        org_id: None,
        access_level: request.access_level,
        active: Some(true),
        deleted: Some(false),
        created_at: Some(Utc::now()),
        expires_at: request.expires_at,
        deleted_at: None,
        deleted_by: None,
    };

    let api_key_usage = ApiKeyUsage {
        id: None,
        api_key: key.clone(),
        usage: request.initial_usage,
        usage_type: request.usage_type.clone(),
        created_at: Some(Utc::now()),
    };

    let api_key_limit = ApiKeyLimit {
        id: None,
        api_key: key.clone(),
        usage_limit: request.usage_limit,
        usage_type: request.usage_type,
        created_at: Some(Utc::now()),
    };

    let transaction = pg_client.transaction().await?;

    let insert_api_key = r#"
    INSERT INTO api_keys (key, user_id, dataset_id, org_id, access_level, active, deleted, created_at, expires_at, deleted_at, deleted_by)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#;

    let insert_api_key_usage = r#"
    INSERT INTO api_key_usage (api_key, usage, usage_type, created_at)
    VALUES ($1, $2, $3, $4)
    "#;

    let insert_api_key_limit = r#"
    INSERT INTO api_key_limit (api_key, usage_limit, usage_type, created_at)
    VALUES ($1, $2, $3, $4)
    "#;

    transaction
        .execute(
            insert_api_key,
            &[
                &api_key.key,
                &api_key.user_id,
                &api_key.dataset_id,
                &api_key.org_id,
                &api_key.access_level.as_ref().map(|al| al.to_string()),
                &api_key.active,
                &api_key.deleted,
                &api_key.created_at,
                &api_key.expires_at,
                &api_key.deleted_at,
                &api_key.deleted_by,
            ],
        )
        .await?;

    transaction
        .execute(
            insert_api_key_usage,
            &[
                &api_key_usage.api_key,
                &api_key_usage.usage,
                &api_key_usage.usage_type.as_ref().map(|ut| ut.to_string()),
                &api_key_usage.created_at,
            ],
        )
        .await?;

    transaction
        .execute(
            insert_api_key_limit,
            &[
                &api_key_limit.api_key,
                &api_key_limit.usage_limit,
                &api_key_limit.usage_type.as_ref().map(|ut| ut.to_string()),
                &api_key_limit.created_at,
            ],
        )
        .await?;

    transaction.commit().await?;

    Ok(key)
}

// // Update the main function to use the new parameters
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let new_key = put_api_key(
//         Some("akshat".to_string()),
//         None,
//         Some("preternatural".to_string()),
//         AccessLevel::ADMIN,
//         None,
//         Some(0),
//         Some(1000),
//         UsageType::FREE,
//     )
//     .await?;
//     println!("New API key created: {}", new_key);
//     Ok(())
// }
