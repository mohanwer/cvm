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



/// # CVM Client Monitor
///
/// A library that keeps the latest version of an application running on a client.
/// This library will check the CVM server for the latest version, run it, and poll the server
/// until a new version is found. Once a new version is found, the old version shuts down, and
/// the new version is started.
impl CvmClientMonitor {

    // TODO: Pass in http client so that we can pass in a mock http client.
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

    /// Starts a child process for the app and while the parent process polls for a new version.
    /// When a new version is found, the running child process will shut down. A new process
    /// will be started with the latest version.
    pub async fn run_and_remain_alive(&mut self) -> Result<()> {
        loop {
            self.run_latest_until_version_outdated().await?;
        }
    }

    /// Runs a specific version of the app until a new version is found. This is particularly
    /// useful in case we want to test version handling or want to start off an earlier version.
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

    /// Runs the latest version of the app until a new version is found. If a new version is found
    /// then the current version is gracefully shutdown and the result of the run is returned.
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

    /// Calls the server to get the latest version number. If it is not currently on the file system
    /// then it is downloaded.
    async fn get_latest_file_path(&mut self) -> Result<PathBuf> {
        let latest_version_response = self.http_client.check_latest().await?;
        let file_name = &latest_version_response.get_file_name();
        // TODO: use named error in place of unwrap.
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

    /// Starts a separate process to run the application. While the application is running, the
    /// parent process polls for a new version. If a new version is found, the child process is
    /// gracefully shutdown.
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
            // It's possible a lifetime duration was passed so this may not always be true especially
            // during integration tests.
            let _ = graceful_shutdown(current_running_version)?;
        }
        Ok(())
    }

    /// Polls server on an interval for new version. When a new version is found true is returned.
    /// If life_time_duration is set, the polling will end at the end of the specified lifetime.
    /// life_time_duration is primarily used to allow the application to halt during integration
    /// tests.
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

/// Starts the process found at the path_buf. Once started, the application will be checked
/// up to three times to ensure it does not exit early.
pub async fn start_process(path_buf: &PathBuf) -> Result<Child> {
    // Set file permissions to 777
    let mut perms = std::fs::metadata(&path_buf)
        .map_err(map_io_error)?
        .permissions();
    perms.set_mode(0o777);

    std::fs::set_permissions(&path_buf, perms).map_err(map_io_error)?;

    let mut child = Command::new(path_buf)
        .spawn()
        // TODO: Use named error
        .expect("Failed to start the process");
    let mut cnt_proc_checked = 0;

    // TODO: Move 3 to constant.
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

/// Shuts down an application using Sigterm and waits for the shutdown to occur.
/// The application may need to be drained for in flight messages which is why we wait for shutdown.
pub fn graceful_shutdown(child: Child) -> Result<bool> {
    let sigterm = ctrlc::Signal::SIGTERM.to_string();
    let mut kill = Command::new("kill")
        .args(["-s", &sigterm, &child.id().to_string()])
        .spawn()
        // TODO: Use named error
        .expect("Unable to send command");

    // TODO: Specify a wait time before issuing a kill. Use named error.
    kill.wait().expect("couldn't wait");
    Ok(true)
}

/// Extracts the version number from the file name. Assumes the file name will be in the format
/// file_name_0.3.2
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
