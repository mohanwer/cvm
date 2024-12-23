use std::fmt;
use std::fmt::Formatter;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum CvmError {
    UnableToCompareVersions { old: String, new: String },
    LatestVersionNotFound { current_version: String },
    TemporaryDirectoryFailedToCreate { message: String },
    NewVersionDownloadFailed { message: String },
    ProcessExitEarly { status: ExitStatus },
    ProcessFailedToStart { message: String },
    ShutdownFailed { message: String },
    ServerUnreachable { message: String },
    SerializingClientDetailsFailed { message: String },
}

impl fmt::Display for CvmError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CvmError::UnableToCompareVersions { old, new } => write!(
                f,
                "Unable to compare old and new version: old version: {} | new version: {}",
                old, new
            ),
            CvmError::LatestVersionNotFound { current_version } => write!(
                f,
                "Latest version not found: currently running: {}",
                current_version
            ),
            CvmError::TemporaryDirectoryFailedToCreate { message } => {
                write!(f, "Unable to create temporary directory: {}", message)
            }
            CvmError::NewVersionDownloadFailed { message } => {
                write!(f, "Unable to download new version: {}", message)
            }
            CvmError::ProcessExitEarly { status } => {
                write!(f, "Exited with status code: {}", status)
            }
            CvmError::ProcessFailedToStart { message } => {
                write!(f, "Unable to start process: {}", message)
            }
            CvmError::ServerUnreachable { message } => {
                write!(f, "Update server error: {}", message)
            }
            CvmError::SerializingClientDetailsFailed { message } => {
                write!(f, "Unable to serialize client details: {}", message)
            }
            CvmError::ShutdownFailed { message } => {
                write!(f, "Shutdown process failed: {}", message)
            }
        }
    }
}

pub fn map_io_error(io_err: std::io::Error) -> CvmError {
    CvmError::TemporaryDirectoryFailedToCreate {
        message: io_err.to_string(),
    }
}

pub fn map_reqwuest_error(reqwuest_err: reqwest::Error) -> CvmError {
    println!("{}", reqwuest_err.to_string());
    CvmError::NewVersionDownloadFailed {
        message: reqwuest_err.to_string(),
    }
}

pub fn map_serialize_error(serde_err: serde_json::Error) -> CvmError {
    CvmError::SerializingClientDetailsFailed {
        message: serde_err.to_string(),
    }
}

pub type Result<T> = std::result::Result<T, CvmError>;
