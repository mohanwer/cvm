use crate::config::Config;
use crate::errors::{map_io_error, map_serialize_error, Result};
use crate::map_reqwuest_error;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::Builder;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct LatestVersionResponse {
    pub build_id: String,
    pub version: String,
    pub url: String,
    pub update_required: bool,
}

impl LatestVersionResponse {
    pub fn get_file_name(&self) -> String {
        self.url.split('/').last().unwrap().to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientDetails {
    pub client_id: String,
    pub app_id: String,
    pub current_running_version: String,
    pub architecture: String,
}

pub struct CvmHttpClient {
    pub client_details: ClientDetails,
    pub latest_version_url: Url,
    report_success_url: Url,
    report_failure_url: Url,
    client: reqwest::Client,
}

impl CvmHttpClient {
    pub fn new(config: Config, version: &str) -> CvmHttpClient {
        let latest_version_url =
            Url::from_str(format!("{}/application/latest", config.cvm_server_url).as_str())
                .expect("invalid latest_version_url");
        let report_success_url =
            Url::from_str(format!("{}/client/success", config.cvm_server_url).as_str())
                .expect("invalid report_success_url");
        let report_failure_url =
            Url::from_str(format!("{}/client/failure", &config.cvm_server_url).as_str())
                .expect("invalid report_failure_url");
        let client = reqwest::Client::new();
        let client_details = ClientDetails {
            client_id: config.client_id,
            app_id: config.app_id,
            current_running_version: version.to_string(),
            architecture: config.architecture,
        };

        CvmHttpClient {
            client_details,
            latest_version_url,
            report_success_url,
            client,
            report_failure_url,
        }
    }

    pub fn set_version(&mut self, version: &str) {
        self.client_details.current_running_version = version.to_string();
    }

    pub async fn check_latest(&mut self) -> Result<LatestVersionResponse> {
        let payload = serde_json::to_value(&self.client_details).map_err(map_serialize_error)?;
        let response = self
            .client
            .post(&self.latest_version_url.to_string())
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwuest_error)?;
        println!("{}", &payload);
        let result = response
            .error_for_status()
            .unwrap()
            .json::<LatestVersionResponse>()
            .await
            .map_err(map_reqwuest_error)?;

        if !result
            .version
            .eq(&self.client_details.current_running_version)
        {
            self.set_version(&result.version);
        }

        Ok(result)
    }

    pub async fn report_healthy(&mut self) -> Result<()> {
        let payload = serde_json::to_value(&self.client_details).map_err(map_serialize_error)?;
        let response = self
            .client
            .post(&self.report_success_url.to_string())
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwuest_error)?;

        response.error_for_status().map_err(map_reqwuest_error)?;

        Ok(())
    }

    pub async fn report_failure(&mut self) -> Result<()> {
        let payload = serde_json::to_value(&self.client_details).map_err(map_serialize_error)?;
        let response = self
            .client
            .post(&self.report_failure_url.to_string())
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwuest_error)?;

        response.error_for_status().map_err(map_reqwuest_error)?;

        Ok(())
    }

    pub async fn download_version(&mut self, url: &str) -> Result<PathBuf> {
        Builder::new()
            .prefix("cvm_tmp_downloads")
            .tempdir()
            .map_err(map_io_error)?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(map_reqwuest_error)?;
        let file_path: PathBuf;
        let mut dest = {
            let file_name = response
                .url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("tmp.bin");
            file_path = current_dir().unwrap().join(file_name);
            File::create(file_name).map_err(map_io_error)?
        };

        let content = response.bytes().await.map_err(map_reqwuest_error)?;
        let _ = dest.write_all(&content);

        Ok(file_path)
    }
}
