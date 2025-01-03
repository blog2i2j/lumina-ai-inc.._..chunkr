use chrono::Utc;
use core::configs::pdfium_config::Config as PdfiumConfig;
use core::configs::worker_config::Config as WorkerConfig;
use core::models::chunkr::pipeline::Pipeline;
use core::models::chunkr::task::Status;
use core::models::chunkr::task::TaskPayload;
use core::models::rrq::queue::QueuePayload;
use core::pipeline::convert_to_images;
use core::pipeline::pages;
use core::pipeline::update_metadata;
use core::utils::clients::initialize_clients;
use core::utils::rrq::consumer::consumer;
use core::utils::storage::services::download_to_tempfile;

async fn execute_step(
    step: &str,
    pipeline: &mut Pipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Executing step: {}", step);
    let start = std::time::Instant::now();
    let result = match step {
        "convert_to_images" => convert_to_images::process(pipeline).await,
        "pages" => pages::process(pipeline).await,
        "update_metadata" => update_metadata::process(pipeline).await,
        _ => Err(format!("Unknown function: {}", step).into()),
    }?;
    let duration = start.elapsed();
    println!(
        "Step {} took {:?} with page count {:?}",
        step,
        duration,
        pipeline.page_count.unwrap_or(0)
    );
    pipeline
        .update_status(result.0.clone(), Some(result.1.clone().unwrap_or_default()))
        .await?;
    Ok(())
}

fn orchestrate_task() -> Vec<&'static str> {
    vec!["update_metadata", "convert_to_images", "pages"]
}

pub async fn process(payload: QueuePayload) -> Result<(), Box<dyn std::error::Error>> {
    let mut pipeline = Pipeline::new();
    let result: Result<(), Box<dyn std::error::Error>> = (async {
        let task_payload: TaskPayload = serde_json::from_value(payload.payload)?;
        let input_file = download_to_tempfile(&task_payload.input_location, None)
            .await
            .map_err(|e| {
                println!("Failed to download input file: {:?}", e);
                e
            })?;

        pipeline.init(input_file, task_payload).await?;

        for step in orchestrate_task() {
            execute_step(step, &mut pipeline).await?;
            if pipeline.status.clone().unwrap().clone() != Status::Processing {
                return Ok(());
            }
        }

        // TODO: Change status to succeeded after development
        pipeline
            .update_status(Status::Failed, Some("Task succeeded".to_string()))
            .await?;

        Ok(())
    })
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let message = match payload.attempt >= payload.max_attempts {
                true => "Task failed".to_string(),
                false => format!("Retrying task {}/{}", payload.attempt, payload.max_attempts),
            };
            pipeline
                .update_status(Status::Failed, Some(message))
                .await?;
            Err(e)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting task processor");
    let config = WorkerConfig::from_env()?;
    PdfiumConfig::from_env()?.ensure_binary().await?;
    initialize_clients().await;
    consumer(process, config.queue_task, 1, 2400).await?;
    Ok(())
}
