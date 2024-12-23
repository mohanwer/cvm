use sqlx::{Acquire, PgPool, Postgres};
use uuid::{Uuid};
use chrono::prelude::*;
use sqlx::pool::PoolConnection;
use sqlx::postgres::any::AnyConnectionBackend;
use sqlx::postgres::PgPoolOptions;
use crate::app_store::AppStoreError::{BuildCreationError, RecordCreationError, RowNotFound, TransactionFailure, VersionCreationError, ConnectionError};
use crate::config::{Config, CONFIG};
use crate::db_commands::{DELETE_APPLICATION, DELETE_CLIENT_BY_ID, INSERT_APPLICATION_BUILD, INSERT_APPLICATION_VERSION, INSERT_CLIENT, INSERT_INTO_APPLICATION, QUERY_ADVISORY_LOCK, QUERY_APPLICATION_BUILD_VERSION, QUERY_APPLICATION_BY_ID, QUERY_APPLICATION_VERSION, QUERY_CLIENT, QUERY_LATEST_BUILD_VERSION, UPDATE_APPLICATION_BUILD_FAILURE, UPDATE_APPLICATION_BUILD_SUCCESS, UPDATE_CLIENT};

#[derive(Debug)]
pub enum AppStoreError {
    RowNotFound { id: String, message: String },
    ConnectionError,
    RecordCreationError { message: String },
    TransactionFailure { message: String },
    VersionCreationError { message: String },
    BuildCreationError { message: String },
}

impl std::fmt::Display for AppStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowNotFound { id, message } => { write!(f, "Client {} not found: {}", id, message) },
            ConnectionError => { write!(f, "Connection error") },
            RecordCreationError { message } => { write!(f, "Failed to create client: {}", message) },
            TransactionFailure { message } => { write!(f, "Transaction failure: {}", message) },
            VersionCreationError { message } => { write!(f, "Failed to create version: {}", message) },
            BuildCreationError { message } => { write!(f, "Failed to build client: {}", message) }
        }
    }
}

pub type Result<T> = std::result::Result<T, AppStoreError>;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Client {
    id: Uuid,
    app_id: Uuid,
    updated_at: DateTime<Utc>,
    version: String,
    enabled: bool,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ApplicationVersion {
    pub id: Uuid,
    pub app_id: Uuid,
    pub version: String,
    pub latest: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct ApplicationBuild {
    pub id: Uuid,
    pub app_version_id: Uuid,
    pub build_version: String,
    pub success_count: i32,
    pub failed_count: i32,
    pub url: String,
    pub disabled: bool
}

#[derive(sqlx::FromRow, Debug)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

pub struct AppStore {
    app_config: &'static Config,
    connection_pool: PoolConnection<Postgres>
}


impl AppStore {

    pub async fn from_config(app_config: &'static Config) -> Result<Self> {
        let pool = PgPoolOptions::new().connect(&app_config.db_url).await.expect("Database connection failed");
        let connection_pool = pool.acquire().await.map_err(|err| { TransactionFailure{message: err.to_string()}})?;
        let mut store = AppStore { app_config, connection_pool };
        store.begin().await.expect("Database connection failed");
        Ok(store)
    }

    pub async fn from_pg_pool(app_config: &'static Config, pool: &PgPool) -> Result<Self> {
        let connection_pool = pool.acquire().await.map_err(|err| { TransactionFailure{message: err.to_string()}})?;
        let mut store = AppStore { app_config, connection_pool };
        store.begin().await.expect("Database connection failed");
        Ok(store)
    }

    pub async fn from_pool_connection(app_config: &'static Config, connection_pool: PoolConnection<Postgres>) -> Result<Self> {
        let mut store = AppStore { app_config, connection_pool };
        store.begin().await.expect("Database connection failed");
        Ok(store)
    }

    pub async fn begin(&mut self) -> Result<()> {
        self.connection_pool
            .begin()
            .await
            .expect("Database connection failed");
        Ok(())
    }

    pub async fn end_transaction(&mut self) -> Result<()> {
        self.connection_pool.commit().await.map_err(|err| { TransactionFailure{message: err.to_string()}})
    }

    pub async fn create_client(&mut self, app_id: Uuid, build_ver: &str) -> Result<Client> {
        sqlx::query_as(INSERT_CLIENT)
            .bind(app_id)
            .bind(&self.app_config.default_version)
            .bind(build_ver)
            .fetch_one(&mut *self.connection_pool)
            .await.map_err(|err| {
                RecordCreationError { message: err.to_string() }
            } )
    }
    
    pub async fn delete_client(&mut self, client_id: Uuid) -> Result<()> {
        sqlx::query(DELETE_CLIENT_BY_ID)
            .bind(client_id)
            .execute(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: client_id.to_string(),
                    message: err.to_string()
                }
            })?;
        Ok(())
    }

    pub async fn get_application_build(
        &mut self,
        app_id: Uuid,
        version: &str,
        architecture: &str,
        for_update: bool
    ) -> Result<ApplicationBuild> {
        let mut query: String = QUERY_APPLICATION_BUILD_VERSION.to_string();

        if for_update {
            query += " FOR UPDATE"
        }

        sqlx::query_as::<_, ApplicationBuild>(&query)
            .bind(app_id)
            .bind(version)
            .bind(architecture)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: format!("App Version ID: {}, Architecture: {}", app_id, architecture),
                    message: err.to_string(),
                }
            })
    }

    pub async fn increment_success_count_by_id(
        &mut self,
        id: Uuid,
    ) -> Result<()> {
        sqlx::query(UPDATE_APPLICATION_BUILD_SUCCESS)
            .bind(id)
            .execute(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: format!("App build ID: {}", id),
                    message: err.to_string(),
                }
            })?;
        Ok(())
    }

    pub async fn increment_failure_count_by_id(
        &mut self,
        id: Uuid,
    ) -> Result<()> {
        sqlx::query(UPDATE_APPLICATION_BUILD_FAILURE)
            .bind(id)
            .bind(id)
            .execute(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: format!("App build ID: {}", id),
                    message: err.to_string(),
                }
            })?;
        Ok(())
    }
    
    pub async fn update_client_version(&mut self, client_id: Uuid, new_version: &str) -> Result<()> {
        sqlx::query(UPDATE_CLIENT)
            .bind(new_version)
            .bind(client_id)
            .execute(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: err.to_string(),
                    message: err.to_string()
                }
            })?;
        Ok(())
    }
    
    pub async fn get_client_by_id(&mut self, client_id: Uuid) -> Result<Client> {
        sqlx::query_as::<_, Client>(QUERY_CLIENT)
            .bind(client_id)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: err.to_string(),
                    message: err.to_string()
                }
            })
    }

    pub async fn create_application_version(&mut self, app_id: Uuid, version: &str, latest: bool) -> Result<ApplicationVersion> {
        sqlx::query_as::<_, ApplicationVersion>(INSERT_APPLICATION_VERSION)
            .bind(app_id)
            .bind(version)
            .bind(latest)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                VersionCreationError {
                    message: err.to_string(),
                }
            })
    }

    pub async fn get_application_version_by_id(&mut self, id: Uuid) -> Result<ApplicationVersion> {
        sqlx::query_as::<_, ApplicationVersion>(QUERY_APPLICATION_VERSION)
            .bind(id)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                VersionCreationError {
                    message: err.to_string(),
                }
            })
    }

    pub async fn create_application_build(
        &mut self,
        app_version_id: Uuid,
        build_version: &str,
        url: &str,
    ) -> Result<ApplicationBuild> {
        sqlx::query_as::<_, ApplicationBuild>(INSERT_APPLICATION_BUILD)
            .bind(url)
            .bind(build_version)
            .bind(app_version_id)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                BuildCreationError {
                    message: err.to_string(),
                }
            })
    }

    pub async fn get_latest_application_version_build(
        &mut self,
        app_id: Uuid,
        build_version: &str,
    ) -> Result<ApplicationBuild> {
        sqlx::query_as::<_, ApplicationBuild>(&QUERY_LATEST_BUILD_VERSION)
            .bind(build_version)
            .bind(app_id)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| {
                RowNotFound {
                    id: format!("App ID: {}, Version: {}", app_id, build_version),
                    message: err.to_string(),
                }
            })
    }
    

    pub async fn create_application(
        &mut self,
        name: &str,
        description: &str
    ) -> Result<Application> {
        sqlx::query_as::<_, Application>(INSERT_INTO_APPLICATION)
            .bind(name)
            .bind(description)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|e| {
                RecordCreationError {
                    message: e.to_string(),
                }
            })
    }

    pub async fn get_application_by_id(
        &mut self,
        app_id: Uuid,
    ) -> Result<Application> {
        sqlx::query_as::<_, Application>(QUERY_APPLICATION_BY_ID)
            .bind(app_id)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|err| RowNotFound { id: app_id.to_string(), message: err.to_string() })
    }

    pub async fn delete_application_by_id(
        &mut self,
        app_id: Uuid,
    ) -> Result<u64> {
        sqlx::query(DELETE_APPLICATION)
            .bind(app_id)
            .execute(&mut *self.connection_pool)
            .await
            .map(|res| res.rows_affected())
            .map_err(|e| TransactionFailure {
                message: e.to_string(),
            })
    }


    pub async fn query_advisory_lock(&mut self, lock_id: Uuid) -> Result<bool> {
        let uuid = lock_id.to_string();
        sqlx::query_scalar(QUERY_ADVISORY_LOCK)
            .bind(uuid)
            .fetch_one(&mut *self.connection_pool)
            .await
            .map_err(|e| TransactionFailure {
                message: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Executor};
    use crate::config::{DB_HOST_DEFAULT, DB_PWD_DEFAULT, DB_URL_TEMPLATE, DB_USER_DEFAULT, DEFAULT_VERSION};
    use once_cell::sync::Lazy;

    static TEST_CONFIG: Lazy<Config> = Lazy::new(|| Config {
        db_url: "postgresql://postgres@127.0.0.1:5432/client_version_manager_test".to_string(),
        db_name: "client_version_manager_test".to_string(),
        db_host: DB_HOST_DEFAULT.to_string(),
        db_user: DB_USER_DEFAULT.to_string(),
        db_pwd: DB_PWD_DEFAULT.to_string(),
        content_url: DB_URL_TEMPLATE.to_string(),
        default_version: DEFAULT_VERSION.to_string(),
    });

    async fn setup() -> Result<(AppStore)> {
        let store = AppStore::from_config(&TEST_CONFIG).await?;
        Ok( store )
    }

    macro_rules! setup_context {
        () => {
            setup().await.expect("Failed to setup database pool")
        };
    }

    #[tokio::test]
    async fn test_new_client_store() {
        let mut store = setup_context!();
        assert!(store.connection_pool.execute("select 1;").await.is_ok());
    }

    #[tokio::test]
    async fn test_create_client() {
        let mut store = setup_context!();
        let app = store.create_application(&"abc", "abcd").await.unwrap();
        let result = store.create_client(app.id,"0.0.1").await;
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.version, DEFAULT_VERSION);
        assert_eq!(client.enabled, true);
    }

    #[tokio::test]
    async fn test_update_client_version() {
        let mut store = setup_context!();
        let app = store.create_application(&"abc", "abcd").await.unwrap();
        let client = store.create_client(app.id, "0.0.1").await.unwrap();
        let update_result = store.update_client_version(client.id, "0.0.2").await;
        assert!(update_result.is_ok());
        let updated_client = store.get_client_by_id(client.id).await.unwrap();
        assert_eq!(updated_client.version, "0.0.2");
    }

    #[tokio::test]
    async fn test_delete_client() {
        let mut store = setup_context!();
        let app = store.create_application(&"abc", "abcd").await.unwrap();
        let client = store.create_client(app.id,"0.0.1").await.unwrap();
        let delete_result = store.delete_client(client.id).await;
        assert!(delete_result.is_ok());
    }

    #[tokio::test]
    async fn test_create_application_build() {
        let mut store = setup_context!();
        let build_version = "x86_64";
        let url = "http://example.com";
        let app = store.create_application(&"abc", "abcd").await.unwrap();
        let app_version = store.create_application_version(app.id, "0.0.1", true).await.unwrap();
        let build = store.create_application_build(app_version.id, build_version, url).await.unwrap();
        assert_eq!(build.url, url);
    }
}
