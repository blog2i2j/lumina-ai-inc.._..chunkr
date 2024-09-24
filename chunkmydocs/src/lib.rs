use actix_cors::Cors;
use actix_multipart::form::MultipartFormConfig;
use actix_multipart::MultipartError;
use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::{ web, App, HttpServer };
use diesel_migrations::{ embed_migrations, EmbeddedMigrations, MigrationHarness };
use env_logger::Env;

pub mod task;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

use middleware::auth::AuthMiddlewareFactory;
use routes::user::get_or_create_user;
use routes::health::health_check;
use routes::task::{ create_extraction_task, get_task_status };
use routes::tasks::get_tasks_status;
use routes::usage::get_usage;
use routes::stripe::{ create_setup_intent, stripe_webhook, create_stripe_session, get_user_invoices, get_invoice_detail, get_monthly_usage };
use utils::db::deadpool_postgres;
use utils::storage::config_s3::create_client;
use utils::server::admin_user::get_or_create_admin_user;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

fn run_migrations(url: &str) {
    use diesel::prelude::*;

    let mut conn = diesel::pg::PgConnection::establish(url).expect("Failed to connect to database");
    conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");

    println!("Migrations ran successfully");
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Chunkr API",
        description = "API service for document layout analysis and chunking to convert document into RAG/LLM-ready data.",
        contact(name = "Lumina", url = "https://lumina.sh", email = "ishaan@lumina.sh"),
        version = "0.0.0"
    ),
    servers(
        (url = "https://chunkr.ai", description = "Production server"),
    ),
    paths(
        routes::health::health_check,   
        routes::task::create_extraction_task,
        routes::task::get_task_status,
    ),
    components(
        schemas(
            models::server::extract::UploadForm,
            models::server::extract::Configuration,
            models::server::extract::Model,
            models::server::task::TaskResponse,
            models::server::task::Status,
            models::server::segment::SegmentType,
            models::server::segment::Segment,
            models::server::segment::Chunk,
            models::server::segment::OCRResult,
            models::server::segment::BoundingBox,
            models::server::extract::OcrStrategy,
        )
    ),
    tags(
        (name = "Health", description = "Endpoint for checking the health of the service."),
        (name = "Task", description = "Endpoints for creating and getting task status")
    )
)]
pub struct ApiDoc;

#[get("/openapi.json")]
pub async fn get_openapi_spec_handler() -> impl actix_web::Responder {
    web::Json(ApiDoc::openapi())
}

pub fn main() -> std::io::Result<()> {

    actix_web::rt::System::new().block_on(async move {
        let pg_pool = deadpool_postgres::create_pool();
        let s3_client = create_client().await.expect("Failed to create S3 client");
        run_migrations(&std::env::var("PG__URL").expect("PG__URL must be set in .env file"));
        get_or_create_admin_user(&pg_pool).await.expect("Failed to create admin user");

        fn handle_multipart_error(err: MultipartError, _: &HttpRequest) -> Error {
            println!("Multipart error: {}", err);
            Error::from(err)
        }

        let max_size: usize = std::env
            ::var("MAX_TOTAL_LIMIT")
            .unwrap_or_else(|_| "10485760".to_string()) // Default to 10 MB if not set
            .parse()
            .expect("MAX_TOTAL_LIMIT must be a valid usize");
        let max_memory_size: usize = std::env
            ::var("MAX_MEMORY_LIMIT")
            .unwrap_or_else(|_| "10485760".to_string()) // Default to 10 MB if not set
            .parse()
            .expect("MAX_MEMORY_LIMIT must be a valid usize");
        let timeout: usize = std::env
            ::var("TIMEOUT")
            .unwrap_or_else(|_| "600".to_string())
            .parse()
            .expect("TIMEOUT must be a valid usize");
        let timeout = std::time::Duration::from_secs(timeout.try_into().unwrap());

        env_logger::init_from_env(Env::default().default_filter_or("info"));

        HttpServer::new(move || {
            let mut app = App::new()
                .wrap(Cors::permissive())
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .app_data(web::Data::new(pg_pool.clone()))
                .app_data(web::Data::new(s3_client.clone()))
                .app_data(
                    MultipartFormConfig::default()
                        .total_limit(max_size)
                        .memory_limit(max_memory_size)
                        .error_handler(handle_multipart_error)
                )
                .service(Redoc::with_url("/redoc", ApiDoc::openapi()))
                .route("/", web::get().to(health_check))
                .route("/health", web::get().to(health_check))
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}").url("/docs/openapi.json", ApiDoc::openapi()),
                );

            let api_scope = web
                ::scope("/api/v1")
                .wrap(AuthMiddlewareFactory)
                .route("/user", web::get().to(get_or_create_user))
                .route("/task", web::post().to(create_extraction_task))
                .route("/task/{task_id}", web::get().to(get_task_status))
                .route("/tasks", web::get().to(get_tasks_status))
                .route("/usage", web::get().to(get_usage))
                .route("/usage/monthly", web::get().to(get_monthly_usage));

            if std::env::var("STRIPE__API_KEY").is_ok() {
                app = app.route("/stripe/webhook", web::post().to(stripe_webhook));

                let stripe_scope = web::scope("/stripe")
                    .wrap(AuthMiddlewareFactory)
                    .route("/create-setup-intent", web::get().to(create_setup_intent))
                    .route("/create-session", web::get().to(create_stripe_session))
                    .route("/invoices", web::get().to(get_user_invoices))
                    .route("/invoice/{invoice_id}", web::get().to(get_invoice_detail));

                app = app.service(stripe_scope);
            }
            

            app.service(api_scope)
        })
            .bind("0.0.0.0:8000")?
            .keep_alive(timeout)
            .run().await
    })
}
