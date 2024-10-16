use crate::models::rrq::queue::QueuePayload;
use crate::models::server::extract::{ ExtractionPayload, OcrStrategy };
use crate::models::server::segment::{ Chunk, Segment, SegmentType };
use crate::models::server::task::Status;
use crate::utils::db::deadpool_postgres::create_pool;
use crate::utils::services::ocr::download_and_ocr;
use crate::utils::storage::config_s3::create_client;
use crate::utils::storage::services::{ download_to_tempfile, upload_to_s3 };
use crate::utils::workers::log::log_task;
use chrono::Utc;
use futures::future::try_join_all;
use std::io::{ Read, Write };
use tempfile::NamedTempFile;

pub fn filter_segment(segment: &Segment, ocr_strategy: OcrStrategy) -> bool {
    if segment.image.is_none() {
        return false;
    }
    match ocr_strategy {
        OcrStrategy::Off => false,
        OcrStrategy::All => true,
        OcrStrategy::Auto => {
            match segment.segment_type {
                SegmentType::Table => true,
                SegmentType::Picture => true,
                _ => {
                    if segment.content.is_empty() { true } else { false }
                }
            }
        }
    }
}

pub async fn process(payload: QueuePayload) -> Result<(), Box<dyn std::error::Error>> {
    let s3_client = create_client().await?;
    let reqwest_client = reqwest::Client::new();
    let extraction_payload: ExtractionPayload = serde_json::from_value(payload.payload)?;
    let task_id = extraction_payload.task_id.clone();
    let pg_pool = create_pool();

    let result: Result<(), Box<dyn std::error::Error>> = (async {
        log_task(
            task_id.clone(),
            Status::Processing,
            Some("OCR started".to_string()),
            None,
            &pg_pool
        ).await?;

        let chunks_file: NamedTempFile = download_to_tempfile(
            &s3_client,
            &reqwest_client,
            &extraction_payload.output_location,
            None
        ).await?;

        let mut file_contents = String::new();
        chunks_file.as_file().read_to_string(&mut file_contents)?;
        let mut chunks: Vec<Chunk> = serde_json::from_str(&file_contents)?;

        try_join_all(
            chunks.iter_mut().flat_map(|chunk| {
                chunk.segments.iter_mut().filter_map(|segment| {
                    if
                        filter_segment(
                            segment,
                            extraction_payload.configuration.ocr_strategy.clone()
                        )
                    {
                        Some(async {
                            let s3_client = s3_client.clone();
                            let reqwest_client = reqwest_client.clone();
                            let ocr_result = download_and_ocr(
                                &s3_client,
                                &reqwest_client,
                                &segment.image.as_ref().unwrap()
                            ).await;
                            match ocr_result {
                                Ok(ocr_result) => {
                                    segment.ocr = Some(ocr_result);
                                    Ok::<_, Box<dyn std::error::Error>>(())
                                }
                                Err(e) => {
                                    eprintln!("Error processing OCR: {:?}", e);
                                    Ok::<_, Box<dyn std::error::Error>>(())
                                }
                            }
                        })
                    } else {
                        None
                    }
                })
            })
        ).await?;

        let mut output_temp_file = NamedTempFile::new()?;
        output_temp_file.write_all(serde_json::to_string(&chunks)?.as_bytes())?;

        upload_to_s3(
            &s3_client,
            &extraction_payload.output_location,
            &output_temp_file.path()
        ).await?;

        if output_temp_file.path().exists() {
            if let Err(e) = std::fs::remove_file(output_temp_file.path()) {
                eprintln!("Error deleting temporary file: {:?}", e);
            }
        }

        Ok(())
    }).await;

    match result {
        Ok(_) => {
            println!("Task succeeded");
            log_task(
                task_id.clone(),
                Status::Succeeded,
                Some("Task succeeded".to_string()),
                Some(Utc::now()),
                &pg_pool
            ).await?;
            Ok(())
        }
        Err(e) => {
            eprintln!("Error processing task: {:?}", e);
            if payload.attempt >= payload.max_attempts {
                eprintln!("Task failed after {} attempts", payload.max_attempts);
                log_task(
                    task_id.clone(),
                    Status::Failed,
                    Some("OCR failed".to_string()),
                    Some(Utc::now()),
                    &pg_pool
                ).await?;
            }
            Err(e)
        }
    }
}
