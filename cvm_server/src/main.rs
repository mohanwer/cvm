mod app_store;
pub mod db_commands;
pub mod config;
pub mod server;

#[tokio::main]
async fn main() {
    server::start().await;
}
