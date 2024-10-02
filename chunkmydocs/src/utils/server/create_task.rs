use crate::models::auth::auth::UserInfo;
use crate::models::rrq::produce::ProducePayload;
use crate::models::{
    server::extract::{Configuration, ExtractionPayload, SegmentationModel},
    server::task::{Status, TaskResponse},
};
use crate::task::pdf::convert_to_pdf;
use crate::utils::configs::extraction_config::Config;
use crate::utils::db::deadpool_postgres::{Client, Pool};
use crate::utils::rrq::service::produce;
use crate::utils::storage::services::{generate_presigned_url, upload_to_s3};
use actix_multipart::form::tempfile::TempFile;
use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, Utc};
use lopdf::Document;
use mime_guess::MimeGuess;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use uuid::Uuid;

fn detect_file_type(file_path: &Path) -> Result<String, Box<dyn Error>> {
    let guess = MimeGuess::from_path(file_path);
    let mime_type = match guess.first() {
        Some(mime) => mime.to_string(),
        None => "application/octet-stream".to_string(),
    };
    Ok(mime_type)
}

fn is_valid_file_type(
    file_path: &Path,
    original_file_name: &str,
) -> Result<(bool, String), Box<dyn Error>> {
    let extension = Path::new(original_file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let temp_file_name = format!("temp_file.{}", extension);
    std::fs::copy(file_path, &temp_file_name)?;
    let temp_file_path = Path::new(&temp_file_name);
    let mime_type = detect_file_type(temp_file_path)?;
    let is_valid = match mime_type.as_str() {
        "application/pdf" => true,
        "application/vnd.ms-powerpoint" => true,
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => true,
        "application/msword" => true,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => true,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => true,
        "application/vnd.ms-excel" => true,
        "application/vnd.google-apps.document" => true,
        "application/vnd.google-apps.presentation" => true,
        "application/vnd.google-apps.spreadsheet" => true,
        "application/vnd.oasis.opendocument.text" => true,
        "application/vnd.oasis.opendocument.spreadsheet" => true,
        "application/vnd.oasis.opendocument.presentation" => true,
        "application/vnd.oasis.opendocument.graphics" => true,
        "application/vnd.oasis.opendocument.chart" => true,
        _ => false,
    };
    std::fs::remove_file(&temp_file_name)?;
    Ok((is_valid, mime_type.to_string()))
}

async fn produce_extraction_payloads(
    extraction_payload: ExtractionPayload,
) -> Result<(), Box<dyn Error>> {
    let config = Config::from_env()?;

    let queue_name = match extraction_payload.model {
        SegmentationModel::PdlaFast => config.extraction_queue_fast,
        SegmentationModel::Pdla => config.extraction_queue_high_quality,
    };

    let queue_name = queue_name.ok_or_else(|| "Queue name not configured".to_string())?;

    let produce_payload = ProducePayload {
        queue_name,
        publish_channel: None,
        payload: serde_json::to_value(extraction_payload).unwrap(),
        max_attempts: None,
        item_id: Uuid::new_v4().to_string(),
    };

    produce(vec![produce_payload]).await?;

    Ok(())
}
use tempfile::NamedTempFile;
pub async fn create_task(
    pool: &Pool,
    s3_client: &S3Client,
    file: &TempFile,
    user_info: &UserInfo,
    configuration: &Configuration,
) -> Result<TaskResponse, Box<dyn Error>> {
    let task_id = Uuid::new_v4().to_string();

    let user_id = user_info.user_id.clone();
    let api_key = user_info.api_key.clone();
    let model = configuration.model.clone();
    let model_internal = model.to_internal();
    let target_chunk_length = configuration.target_chunk_length.unwrap_or(512);
    let created_at = Utc::now();
    let client: Client = pool.get().await?;
    let config = Config::from_env()?;
    let expiration = config.task_expiration;
    let expiration_time: Option<DateTime<Utc>> = expiration.map(|exp| Utc::now() + exp);
    let bucket_name = config.s3_bucket;
    let ingest_batch_size = config.batch_size;
    let base_url = config.base_url;
    let task_url = format!("{}/api/v1/task/{}", base_url, task_id);

    let file_name = file.file_name.as_deref().unwrap_or("unknown");
    let (is_valid, detected_mime_type) = is_valid_file_type(&file.file.path(), file_name)?;
    if !is_valid {
        return Err(format!("Not a valid file type: {}", detected_mime_type).into());
    }

    let extension = file
        .file_name
        .as_deref()
        .unwrap_or("")
        .split('.')
        .last()
        .unwrap_or("tmp");
    let original_path = PathBuf::from(file.file.path());
    let new_path = original_path.with_extension(extension);

    std::fs::rename(&original_path, &new_path)?;

    let input_path = new_path;

    let output_file = NamedTempFile::new().unwrap();
    let output_path = output_file.path().to_path_buf();

    let result = convert_to_pdf(&input_path, &output_path).await;
    let output_dir = PathBuf::from("output");
    std::fs::create_dir_all(&output_dir).unwrap();
    let final_output_path = output_dir.join("test_output.pdf");

    match result {
        Ok(_) => {
            std::fs::copy(&output_path, &final_output_path).unwrap();
            println!("Test output saved to {:?}", final_output_path);
        }
        Err(e) => {
            panic!("PDF conversion failed: {:?}", e);
        }
    }
    let page_count = match Document::load(&output_path) {
        Ok(doc) => doc.get_pages().len() as i32,
        Err(_) => 0,
    };
    let file_size = file.size;

    let file_name = file
        .file_name
        .as_deref()
        .unwrap_or("unknown.pdf")
        .to_string();
    let file_name = if file_name.ends_with(".pdf") {
        file_name
    } else {
        format!(
            "{}.pdf",
            file_name.trim_end_matches(|c| c == '.' || char::is_alphanumeric(c))
        )
    };
    let input_location = format!("s3://{}/{}/{}/{}", bucket_name, user_id, task_id, file_name);
    let output_extension = model_internal.get_extension();
    let output_location = input_location.replace(".pdf", &format!(".{}", output_extension));
    let image_folder_location =
        format!("s3://{}/{}/{}/{}", bucket_name, user_id, task_id, "images");

    let message = "Task queued".to_string();

    match upload_to_s3(s3_client, &input_location, &output_path).await {
        Ok(_) => {
            let configuration_json = serde_json::to_string(configuration)?;
            match client
                .execute(
                    "INSERT INTO TASKS (
                    task_id, user_id, api_key, file_name, file_size, 
                    page_count, segment_count, expires_at,
                    status, task_url, input_location, output_location, image_folder_location,
                    configuration, message
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
                ) ON CONFLICT (task_id) DO NOTHING",
                    &[
                        &task_id,
                        &user_id,
                        &api_key,
                        &file_name,
                        &(file_size as i64),
                        &page_count,
                        &0i32,
                        &expiration_time,
                        &Status::Starting.to_string(),
                        &task_url,
                        &input_location,
                        &output_location,
                        &image_folder_location,
                        &configuration_json,
                        &message,
                    ],
                )
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    if e.to_string().contains("usage limit exceeded") {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "429 Rate Limit Error: Usage limit exceeded",
                        )));
                    } else {
                        return Err(Box::new(e) as Box<dyn Error>);
                    }
                }
            };

            let extraction_payload = ExtractionPayload {
                user_id: user_id.clone(),
                model: model_internal,
                input_location: input_location.clone(),
                output_location,
                image_folder_location,
                expiration: None,
                batch_size: Some(ingest_batch_size),
                task_id: task_id.clone(),
                target_chunk_length: Some(target_chunk_length),
                configuration: configuration.clone(),
            };

            produce_extraction_payloads(extraction_payload).await?;

            let input_file_url =
                match generate_presigned_url(s3_client, &input_location, None).await {
                    Ok(response) => Some(response),
                    Err(e) => {
                        println!("Error getting input file url: {}", e);
                        return Err("Error getting input file url".into());
                    }
                };

            Ok(TaskResponse {
                task_id: task_id.clone(),
                status: Status::Starting,
                created_at: created_at,
                finished_at: None,
                expires_at: expiration_time,
                output: None,
                input_file_url,
                task_url: Some(task_url),
                message,
                configuration: configuration.clone(),
                file_name: Some(file_name.to_string()),
                page_count: Some(page_count),
            })
        }
        Err(e) => Err(e),
    }
}
