#[path = "../src/updater.rs"]
mod updater;

#[cfg(test)]
mod tests {
    use semver::Version;
    use crate::updater::{check_for_updates, start_updater};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_check_for_updates_no_update() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/0xSolanaceae/discord-imhex/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "v1.0.0",
                "assets": []
            })))
            .mount(&mock_server)
            .await;

        let result = check_for_updates().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // broken :(
    async fn test_start_updater() {
        let mock_server = MockServer::start().await;
    
        Mock::given(method("GET"))
            .and(path("/repos/0xSolanaceae/discord-imhex/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tag_name": "v2.0.0",
                "assets": [{
                    "browser_download_url": "https://example.com/download/v2.0.0"
                }]
            })))
            .mount(&mock_server)
            .await;
    
        let updater_handle = tokio::spawn(async {
            start_updater().await;
        });
    
        sleep(Duration::from_secs(60 * 60 * 4 + 5)).await;
    
        let received_requests = mock_server.received_requests().await.unwrap();
        assert!(!received_requests.is_empty(), "No requests were received by the mock server");
    
        assert!(!updater_handle.is_finished(), "Updater task has finished unexpectedly");
    
        updater_handle.abort();
    }

    #[tokio::test]
    async fn test_check_for_updates_with_update() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/repos/0xSolanaceae/discord-imhex/releases/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"tag_name": "v2.0.0", "assets": [{"browser_download_url": "http://example.com/update.exe"}]}"#))
            .mount(&mock_server)
            .await;

        let result = check_for_updates().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        assert!(v2 > v1);
    }
}