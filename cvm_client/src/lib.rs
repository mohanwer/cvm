pub mod config;
pub mod errors;
pub mod http_client;

use crate::config::{Config, VERSION_ZERO};
use crate::errors::CvmError::{ProcessExitEarly, ProcessFailedToStart};
use crate::errors::Result;
use crate::errors::{map_io_error, map_reqwuest_error};
use crate::http_client::CvmHttpClient;
use std::env::current_dir;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

pub struct CvmClientMonitor {
    pub http_client: CvmHttpClient,
    version_check_poll_interval: Duration,
    life_time_duration: Option<chrono::TimeDelta>,
    life_time_duration_reached: bool,
}

pub struct RunResult {
    pub last_version_ran: String,
    pub latest_version_detected: String,
    pub life_time_duration_reached: bool,
}

impl CvmClientMonitor {
    pub fn new(
        config: Config,
        version_check_poll_interval: Duration,
        life_time_duration: Option<chrono::TimeDelta>,
    ) -> Self {
        CvmClientMonitor {
            http_client: CvmHttpClient::new(config, VERSION_ZERO),
            version_check_poll_interval,
            life_time_duration,
            life_time_duration_reached: false,
        }
    }

    pub async fn run_and_remain_alive(&mut self) -> Result<()> {
        loop {
            self.run_latest_until_version_outdated().await?;
        }
    }

    pub async fn run_specified_version_until_outdated(
        &mut self,
        version_url: &str,
    ) -> Result<RunResult> {
        let version_path = &self.http_client.download_version(version_url).await?;
        let last_version_ran = strip_version_from_file_name(&version_path);
        &self.run_until_new_version_found(version_path).await?;
        let latest_version_detected = self
            .http_client
            .client_details
            .current_running_version
            .clone();

        Ok(RunResult {
            last_version_ran,
            latest_version_detected,
            life_time_duration_reached: self.life_time_duration_reached,
        })
    }

    pub async fn run_latest_until_version_outdated(&mut self) -> Result<RunResult> {
        let latest_path = &self.get_latest_file_path().await?;
        let last_version_ran = strip_version_from_file_name(&latest_path);
        &self.run_until_new_version_found(latest_path).await?;
        let latest_version_detected = self
            .http_client
            .client_details
            .current_running_version
            .clone();

        Ok(RunResult {
            last_version_ran,
            latest_version_detected,
            life_time_duration_reached: self.life_time_duration_reached,
        })
    }

    async fn get_latest_file_path(&mut self) -> Result<PathBuf> {
        let latest_version_response = self.http_client.check_latest().await?;
        let file_name = &latest_version_response.get_file_name();
        let file_path = current_dir().unwrap().join(file_name);
        if file_path.exists() {
            return Ok(file_path);
        }
        let new_path = self
            .http_client
            .download_version(&latest_version_response.url)
            .await?;

        Ok(new_path)
    }

    async fn run_until_new_version_found(&mut self, latest_path: &PathBuf) -> Result<()> {
        let current_running_version: Child;
        match start_process(latest_path).await {
            Ok(child) => {
                current_running_version = child;
                &self.http_client.report_healthy().await;
            }
            Err(err) => {
                &self.http_client.report_failure().await?;
                return Err(err);
            }
        }
        let new_version_found = self
            .poll_until_new_version(self.version_check_poll_interval)
            .await;
        if new_version_found {
            let _ = graceful_shutdown(current_running_version)?;
        }
        Ok(())
    }

    pub async fn poll_until_new_version(&mut self, poll_interval: Duration) -> bool {
        let mut interval = tokio::time::interval(poll_interval);
        let start_poll_time = chrono::Utc::now();

        loop {
            if self.life_time_duration.is_some() {
                let elapsed_time = chrono::Utc::now() - start_poll_time;
                let end_time = &self.life_time_duration.unwrap();
                if end_time.le(&elapsed_time) {
                    println!("Lifetime duration has passed!");
                    self.life_time_duration_reached = true;
                    return false; // Exit the loop if the lifetime duration has passed
                }
            }
            interval.tick().await;
            let latest_version = &self.http_client.check_latest().await;
            match latest_version {
                Ok(response) => {
                    if response.update_required {
                        return true;
                    }
                }
                // For now, we will keep polling in case the server comes back online.
                Err(err) => println!(
                    "Error calling server to check version: {}. Error: {}",
                    &self.http_client.latest_version_url,
                    &err.to_string()
                ),
            }
        }
    }
}

pub async fn start_process(path_buf: &PathBuf) -> Result<Child> {
    // Set file permissions to 777
    let mut perms = std::fs::metadata(&path_buf)
        .map_err(map_io_error)?
        .permissions();
    perms.set_mode(0o777);

    std::fs::set_permissions(&path_buf, perms).map_err(map_io_error)?;

    let mut child = Command::new(path_buf)
        .spawn()
        .expect("Failed to start the process");
    let mut cnt_proc_checked = 0;

    while cnt_proc_checked < 3 {
        match child.try_wait() {
            Ok(Some(status)) => return Err(ProcessExitEarly { status }),
            Ok(None) => {
                println!("status not ready yet, let's really wait");
                cnt_proc_checked += 1;
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(ProcessFailedToStart {
                    message: e.to_string(),
                })
            }
        }
    }
    Ok(child)
}

pub fn graceful_shutdown(child: Child) -> Result<bool> {
    let sigterm = ctrlc::Signal::SIGTERM.to_string();
    let mut kill = Command::new("kill")
        .args(["-s", &sigterm, &child.id().to_string()])
        .spawn()
        .expect("Unable to send command");
    kill.wait().expect("couldn't wait");
    Ok(true)
}

pub fn strip_version_from_file_name(path_buf: &PathBuf) -> String {
    path_buf
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .split("_")
        .last()
        .unwrap()
        .to_string()
}
