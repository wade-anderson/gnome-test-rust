use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("no location data found from any provider")]
    NoLocationFound,
    #[error("forced failure for testing")]
    TestingForcedFailure,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
}

pub struct GeoService {
    pub client: reqwest::Client,
    pub providers: Vec<String>,
}

impl GeoService {
    pub fn new(providers: Vec<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("GnomeTestRust/0.1.0")
            .build()
            .unwrap_or_default();
        Self { client, providers }
    }

    pub async fn fetch_location(&self) -> Result<GeoLocation, GeoError> {
        if std::env::var("GNOME_TEST_FAIL_GEO").is_ok() {
            return Err(GeoError::TestingForcedFailure);
        }
        for provider in &self.providers {
            if let Ok(response) = self.client.get(provider).send().await
                && let Ok(loc) = response.json::<GeoLocation>().await
            {
                return Ok(loc);
            }
        }
        Err(GeoError::NoLocationFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_geo_service_success() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/json");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"latitude": 40.7128, "longitude": -74.0060}"#);
        });

        let service = GeoService::new(vec![server.url("/json")]);
        let loc = service
            .fetch_location()
            .await
            .expect("Should find location");

        assert_eq!(loc.latitude, 40.7128);
        assert_eq!(loc.longitude, -74.0060);
        mock.assert();
    }

    #[tokio::test]
    async fn test_geo_service_fallback() {
        use httpmock::prelude::*;

        let server = MockServer::start();

        // First provider fails
        let mock_fail = server.mock(|when, then| {
            when.method(GET).path("/fail");
            then.status(500);
        });

        // Second provider succeeds
        let mock_success = server.mock(|when, then| {
            when.method(GET).path("/success");
            then.status(200)
                .body(r#"{"latitude": 34.0522, "longitude": -118.2437}"#);
        });

        let service = GeoService::new(vec![server.url("/fail"), server.url("/success")]);

        let loc = service
            .fetch_location()
            .await
            .expect("Should fallback to second provider");

        assert_eq!(loc.latitude, 34.0522);
        assert_eq!(loc.longitude, -118.2437);

        mock_fail.assert();
        mock_success.assert();
    }

    #[tokio::test]
    async fn test_geo_service_total_failure() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/fail");
            then.status(404);
        });

        let service = GeoService::new(vec![server.url("/fail")]);
        let result = service.fetch_location().await;

        assert!(matches!(result, Err(GeoError::NoLocationFound)));
        mock.assert();
    }
}
