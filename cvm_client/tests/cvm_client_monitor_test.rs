#[cfg(test)]
mod cvm_client_monitor {
    use cvm::config::Config;
    use cvm::CvmClientMonitor;
    use std::time::Duration;

    fn get_first_version_url() -> String {
        "https://hello-versioned.s3.us-east-1.amazonaws.com/linux/infinite_hello_0.1.0".to_string()
    }
    fn get_second_version_url() -> String {
        "https://hello-versioned.s3.us-east-1.amazonaws.com/linux/infinite_hello_0.2.0".to_string()
    }

    fn max_run_time() -> chrono::TimeDelta {
        chrono::TimeDelta::new(10, 0).unwrap()
    }

    #[tokio::test]
    async fn it_starts_and_updates_app_to_latest() {
        let first_version = get_first_version_url();
        let config = Config::new().expect("Error while creating config");
        let max_run_time = max_run_time();
        let poll_interval = Duration::from_secs(1);
        let mut cvm_client = CvmClientMonitor::new(config, poll_interval, Some(max_run_time));
        let result = cvm_client
            .run_specified_version_until_outdated(&first_version)
            .await;
        assert!(result.is_ok());
        let last_run_results = result.unwrap();

        // We ran the app with 0.1.0 and it detected 0.2.0 and stopped (which we want for this test).
        assert_eq!(last_run_results.latest_version_detected, "0.2.0");
        assert_eq!(last_run_results.last_version_ran, "0.1.0");
        assert_eq!(last_run_results.life_time_duration_reached, false);
    }

    #[tokio::test]
    async fn it_starts_and_updates_and_runs_latest_version() {
        let config = Config::new().expect("Error while creating config");
        let max_run_time = max_run_time();
        let poll_interval = Duration::from_secs(1);
        let mut cvm_client = CvmClientMonitor::new(config, poll_interval, Some(max_run_time));
        let result = cvm_client.run_latest_until_version_outdated().await;
        assert!(result.is_ok());
        let last_run_results = result.unwrap();

        // We ran the app with 0.1.0, it detected 0.2.0,
        // updated itself to 0.2.0 and ran until the max allowed lifetime.
        assert_eq!(last_run_results.latest_version_detected, "0.2.0");
        assert_eq!(last_run_results.last_version_ran, "0.2.0");
        assert_eq!(last_run_results.life_time_duration_reached, true);
    }
}
