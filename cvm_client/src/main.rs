use cvm::config::Config;
use cvm::CvmClientMonitor;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const POLL_NEW_VERSION_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() {
    start_signal_listener();
    let config = create_config_or_shutdown();
    run_client(config).await;
}

fn create_config_or_shutdown() -> Config {
    let config_result = Config::new();
    if config_result.is_err() {
        eprintln!("Error creating config: {}", config_result.err().unwrap());
        process::exit(1);
    }
    config_result.expect("Error creating config")
}

async fn run_client(config: Config) {
    let mut cvm_client = CvmClientMonitor::new(config, POLL_NEW_VERSION_INTERVAL, None);
    match cvm_client.run_and_remain_alive().await {
        Ok(_) => {
            println!("Gracefully shutting down.");
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Error running client: {}", err);
            process::exit(1);
        }
    }
}

fn start_signal_listener() {
    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Exiting...");
        } else {
            process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");
    println!("Running...");
}
