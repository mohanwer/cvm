#[cfg(test)]
mod http_client_integration_tests {
    use cvm::config::Config;
    use cvm::http_client::*;

    fn get_config() -> Config {
        Config::new().expect("Failed to parse config")
    }

    fn create_http_client() -> CvmHttpClient {
        let cvm_config = get_config();
        CvmHttpClient::new(cvm_config, "0.1.0")
    }

    #[tokio::test]
    async fn it_gets_the_latest_version() {
        let mut http_client = create_http_client();
        let result = http_client.check_latest().await;
        assert!(result.is_ok());
        let result: LatestVersionResponse = result.expect("Failed to fetch latest version");
        assert_eq!(result.version, "0.2.0".to_string());
    }

    #[tokio::test]
    async fn it_reports_success() {
        let mut http_client = create_http_client();
        let result = http_client.report_healthy().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn it_reports_failure() {
        let mut http_client = create_http_client();
        let result = http_client.report_failure().await;
        assert!(result.is_ok());
    }
}
