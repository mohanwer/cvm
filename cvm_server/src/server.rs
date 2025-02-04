use crate::app_store::{AppStore, AppStoreError};
use crate::config::CONFIG;
use axum::routing::post;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Acquire, PgPool};
use std::env::args;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
enum Architecture {
    #[serde(rename = "x86_64-pc-windows-gnu")]
    X86_64Intel,
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    X86_64Linux,
}

impl Architecture {
    fn to_string(&self) -> &str {
        // serde deserializes json with extra quotes.
        match self {
            Architecture::X86_64Intel => "x86_64-pc-windows-gnu",
            Architecture::X86_64Linux => "x86_64-unknown-linux-gnu",
        }
    }
}

#[derive(Deserialize)]
struct CreateApplication {
    name: String,
    description: String,
}

#[derive(Serialize)]
struct Application {
    id: Uuid,
    name: String,
    description: String,
}

#[derive(Deserialize)]
struct CreateApplicationBuild {
    app_id: Uuid,
    version: String,
    architecture: Architecture,
    latest: bool,
    url: String,
}

#[derive(Deserialize)]
struct ClientDetails {
    client_id: Uuid,
    app_id: Uuid,
    current_running_version: String,
    architecture: Architecture,
}

#[derive(Serialize)]
struct LatestVersion {
    build_id: Uuid,
    version: String,
    url: String,
    update_required: bool,
}

/// Starts web server to start listening for cvm clients.
pub async fn start() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create database connection pool that will provide connections to route handlers.
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&CONFIG.db_url)
        .await
        .expect("can't connect to database");

    // Register route handlers
    let app = Router::new()
        .route("/application/create", post(create_application))
        .route("/application/latest", post(get_latest_version))
        .route("/client/success", post(report_build_success))
        .route("/client/failure", post(report_build_failure))
        .route("/health", get(health))
        .with_state(pool);

    // Bind to port and startup server
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// Request context that provides a connection from the connection pool to route handlers.
struct RequestContext(AppStore);

#[async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let mut pool = PgPool::from_ref(state);
        let conn_pool = pool.acquire().await.map_err(internal_error)?;
        let app_store = AppStore::from_pool_connection(&CONFIG, conn_pool)
            .await
            // TODO: Use named error.
            .expect("Failed db");
        Ok(Self(app_store))
    }
}

/// Administrative api for creating a new application. The corresponding api for adding a
/// new version for the application is not yet available.
/// POST:
/// {
///     name: String,
///     description: String
/// }
async fn create_application(
    RequestContext(mut app_store): RequestContext,
    Json(params): Json<CreateApplication>,
) -> Result<Json<Application>, (StatusCode, String)> {
    let app = app_store
        .create_application(&params.name, &params.description)
        .await
        .map_err(app_store_error)?;

    Ok(Json(Application {
        id: app.id,
        name: app.name,
        description: app.description,
    }))
}

/// Returns the latest version of an application based on the client details provided.
/// POST:
/// {
///     client_id: Uuid,
///     app_id: Uuid,
///     current_running_version: String,
///     architecture: Architecture
/// }
///
/// Architecture can be x86_64-pc-windows-gnu or x86_64-unknown-linux-gnu
///
/// Returns
/// {
/// build_id: Uuid,
///     version: String,
///     url: String,
///     update_required: bool,
/// }
async fn get_latest_version(
    RequestContext(mut app_store): RequestContext,
    Json(params): Json<ClientDetails>,
) -> Result<Json<LatestVersion>, (StatusCode, String)> {
    let arch = params.architecture.to_string();
    let app_build = app_store
        .get_latest_application_version_build(params.app_id, arch)
        .await
        .map_err(app_store_error)?;

    let app_version = app_store
        .get_application_version_by_id(app_build.app_version_id)
        .await
        .map_err(app_store_error)?;

    app_store
        .update_client_version(params.client_id, &params.current_running_version)
        .await
        .map_err(app_store_error)?;

    println!("{} {}", app_version.version, params.current_running_version);
    let latest_version = semver::Version::parse(&app_version.version).unwrap();
    let current_version =
        semver::Version::parse(&params.current_running_version).map_err(internal_error)?;
    let update_required = latest_version > current_version;

    Ok(Json(LatestVersion {
        build_id: app_build.id,
        url: app_build.url,
        version: app_version.version,
        update_required,
    }))
}

/// Reports successful build/run for a client.
/// POST:
/// {
///     client_id: Uuid,
///     app_id: Uuid,
///     current_running_version: String,
///     architecture: Architecture
/// }
async fn report_build_success(
    RequestContext(mut app_store): RequestContext,
    Json(params): Json<ClientDetails>,
) -> Result<Json<()>, (StatusCode, String)> {
    let arch = params.architecture.to_string();
    let app_build = app_store
        .get_application_build(params.app_id, &params.current_running_version, arch, true)
        .await
        .map_err(app_store_error)?;

    app_store
        .increment_success_count_by_id(app_build.id)
        .await
        .map_err(app_store_error)?;

    Ok(Json(()))
}

/// Reports failure build/startup
/// POST:
/// {
///     client_id: Uuid,
///     app_id: Uuid,
///     current_running_version: String,
///     architecture: Architecture
/// }
/// TODO: Trigger a roll back on the latest version if too many clients report a failed startup of a version.
async fn report_build_failure(
    RequestContext(mut app_store): RequestContext,
    Json(params): Json<ClientDetails>,
) -> Result<Json<()>, (StatusCode, String)> {
    let arch = params.architecture.to_string();
    let app_build = app_store
        .get_application_build(params.app_id, &params.current_running_version, arch, true)
        .await
        .map_err(app_store_error)?;

    app_store
        .increment_failure_count_by_id(app_build.id)
        .await
        .map_err(app_store_error)?;

    Ok(Json({}))
}

/// Health endpoint for monitoring
async fn health() -> Result<Json<()>, (StatusCode, String)> {
    Ok(Json({}))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn app_store_error(err: AppStoreError) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

fn serialize_architecture(arch: Architecture) -> &'static str {
    match arch {
        Architecture::X86_64Intel => "x86_64-pc-windows-gnu",
        Architecture::X86_64Linux => "x86_64-unknown-linux-gnu",
    }
}
