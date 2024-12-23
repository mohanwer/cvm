use once_cell::sync::Lazy;

pub const DB_HOST_DEFAULT: &str = "127.0.0.1";
pub const DB_USER_DEFAULT: &str = "postgres";
pub const DB_PWD_DEFAULT: &str = "";
pub const DB_NAME: &str = "client_version_manager";
pub const CONTENT_URL_DEFAULT: &str = "https://hello-versioned.s3.us-east-1.amazonaws.com/";
pub const DB_URL_TEMPLATE: &str = "postgresql://{user}@{host}:5432/{db_name}";
pub const DEFAULT_VERSION: &str = "0.0.0";

pub struct Config{
    pub db_host: String,
    pub db_user: String,
    pub db_pwd: String,
    pub db_name: String,
    pub db_url: String,
    pub content_url: String,
    pub default_version: String,
}

fn get_env_var_or<'a>(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let db_host = get_env_var_or("DB_HOST", DB_HOST_DEFAULT);
    let db_user = get_env_var_or("DB_USER", DB_USER_DEFAULT);
    let db_pwd = get_env_var_or("DB_PWD", DB_PWD_DEFAULT);
    let content_url = get_env_var_or("CONTENT_URL", CONTENT_URL_DEFAULT);
    let db_name = if std::env::var("CARGO_TEST").is_ok() {
        format!("{}_test", DB_NAME)
    } else {
        DB_NAME.to_string()
    };
    let db_url = DB_URL_TEMPLATE
        .replace("{user}", &db_user)
        .replace("{password}", &db_pwd)
        .replace("{db_name}", &db_name)
        .replace("{host}", &db_host);
    Config {
        db_host,
        db_user,
        db_pwd,
        db_name,
        db_url,
        content_url,
        default_version: DEFAULT_VERSION.to_string(),
    }
});

pub fn load_config() -> &'static Config {
    &CONFIG
}