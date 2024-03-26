mod login;

use log::info;
use reqwest::header::HeaderValue;
use reqwest::{header, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;

pub struct CephRestfulClientAccess {
    pub host: Url,
    pub username: String,
    pub password: String,
}

pub struct CephRestfulClient {
    client: ClientWithMiddleware,
    access: CephRestfulClientAccess,
    token: Option<String>,
}

#[derive(Debug, Error)]
pub enum CephRestfulClientError {
    #[error("Failed to parse URL")]
    UrlParse(#[from] url::ParseError),
    #[error("Failed to send request")]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error("Failed to send request")]
    Reqwest(#[from] reqwest::Error),
}
static USER_AGENT: &str = concat!("ceph-rbd-metrics/v", env!("CARGO_PKG_VERSION"));
static ACCEPT: &str = "application/vnd.ceph.api.v1.0+json";

impl CephRestfulClient {
    pub fn new(access: CephRestfulClientAccess) -> Self {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        info!("Creating CephRestfulClient...");
        info!("Host: {}", access.host);
        info!("Username: {}", access.username);
        info!("User-Agent: {}", USER_AGENT);
        info!("Accept: {}", ACCEPT);

        CephRestfulClient {
            client: ClientBuilder::new(
                reqwest::Client::builder()
                    .default_headers(
                        [
                            (header::USER_AGENT, HeaderValue::from_static(USER_AGENT)),
                            (header::ACCEPT, HeaderValue::from_static(ACCEPT)),
                        ]
                        .into_iter()
                        .collect(),
                    )
                    .build()
                    .unwrap(),
            )
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build(),
            access,
            token: None,
        }
    }

    fn endpoint(&self, path: &str) -> Result<Url, url::ParseError> {
        self.access.host.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_endpoint() {
        let access = CephRestfulClientAccess {
            host: Url::parse("http://localhost:8000").unwrap(),
            username: "admin".to_string(),
            password: "admin".to_string(),
        };

        let client = CephRestfulClient::new(access);

        assert_eq!(
            client.endpoint("/api/auth/login").unwrap(),
            Url::parse("http://localhost:8000/api/auth/login").unwrap()
        );
    }
}
