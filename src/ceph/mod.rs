mod login;

use std::env;

use log::info;
use reqwest::header::HeaderValue;
use reqwest::{header, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;

use crate::ceph::login::CephApiAuthentication;

#[derive(Clone)]
pub struct CephRestfulClientAccess {
    pub host: Url,
    pub username: String,
    pub password: String,
}

pub struct CephRestfulClient {
    client: ClientWithMiddleware,
    access: CephRestfulClientAccess,
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
    pub async fn from_env() -> Self {
        let access = CephRestfulClientAccess {
            host: Url::parse(&env::var("CEPH_API_ENDPOINT").expect("CEPH_API_ENDPOINT is not set"))
                .expect("CEPH_API_ENDPOINT is not a valid URL"),
            username: env::var("CEPH_API_USERNAME").expect("CEPH_API_USERNAME is not set"),
            password: env::var("CEPH_API_PASSWORD").expect("CEPH_API_PASSWORD is not set"),
        };
        Self::new(access).await
    }

    pub async fn new(access: CephRestfulClientAccess) -> Self {
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
            .with(CephApiAuthentication::new(access.clone()).await)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build(),
            access,
        }
    }

    fn endpoint(&self, path: &str) -> Result<Url, url::ParseError> {
        self.access.host.join(path)
    }
}
