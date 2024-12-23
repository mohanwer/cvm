use crate::config::ConfigError::{ArchitectureNotSupported, OSNotSupported};
use std::fmt::Formatter;

#[derive(Debug)]
pub enum ConfigError {
    OSNotSupported,
    ArchitectureNotSupported,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OSNotSupported => {
                write!(f, "OS is not supported for {}", std::env::consts::OS)
            }
            ArchitectureNotSupported => {
                write!(
                    f,
                    "Architecture is not supported for {}",
                    std::env::consts::ARCH
                )
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, ConfigError>;

pub const DEFAULT_CVM_SERVER_URL: &str = "http://127.0.0.1:3000";
pub const DEFAULT_CLIENT_ID: &str = "f944fc73-e1ca-442a-88bb-642d42a38a6c";
pub const DEFAULT_APP_ID: &str = "50b473ee-35b3-4252-8998-6be4d4130d3a";
pub const DEFAULT_ARCHITECTURE: &str = "x86_64-unknown-linux-gnu";
pub const VERSION_ZERO: &str = "0.0.0";

#[derive(Debug)]
pub struct Config {
    pub cvm_server_url: String,
    pub client_id: String,
    pub app_id: String,
    pub architecture: String,
}

impl Config {
    pub fn new() -> Result<Config> {
        let cvm_server_url = get_env_var_or("CVM_SERVER_URL", DEFAULT_CVM_SERVER_URL);
        let client_id = get_env_var_or("CLIENT_ID", DEFAULT_CLIENT_ID);
        let app_id = get_env_var_or("APP_ID", DEFAULT_APP_ID);
        let architecture = get_architecture()?;

        Ok(Config {
            cvm_server_url,
            client_id,
            app_id,
            architecture,
        })
    }
}

fn get_env_var_or<'a>(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn get_architecture() -> Result<String> {
    // only supporting x86_64 and windows currently.
    // couldn't get mac m1 to build on debian.
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    if arch != "x86_64" {
        Err(ArchitectureNotSupported)
    } else if os == "linux" {
        Ok("x86_64-unknown-linux-gnu".to_string())
    } else if os == "linux" {
        Ok("x86_64-pc-windows-gnu".to_string())
    } else {
        Err(OSNotSupported)
    }
}
