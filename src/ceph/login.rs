use std::sync::Arc;

use log::{info, warn};
use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::{header, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, Middleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use super::{CephRestfulClientAccess, CephRestfulClientError, ACCEPT, USER_AGENT};

pub struct CephApiAuthentication {
    access: CephRestfulClientAccess,
    token: Arc<Mutex<Option<String>>>,
}

impl CephApiAuthentication {
    pub async fn new(access: CephRestfulClientAccess) -> Self {
        let auth = Self {
            access,
            token: Arc::new(Mutex::new(None)),
        };
        auth.refresh_token().await.unwrap();
        auth
    }

    async fn refresh_token(&self) -> Result<(), CephRestfulClientError> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(
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
        .build();

        let body = json!({
            "username": self.access.username,
            "password": self.access.password,
        });

        let response = client
            .post(self.access.host.join("/api/auth")?)
            .json(&body)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token = response.json::<LoginSuccess>().await?;
                info!(
                    "Authentication token was successfully refreshed as {}.",
                    token.username
                );
                self.token.lock().await.replace(token.token);
                Ok(())
            }
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED => {
                panic!("Failed to login.\n {:?}.", response.text().await);
            }
            _ => panic!("Unexpected response: {:?}", response),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for CephApiAuthentication {
    async fn handle(
        &self,
        mut req: reqwest::Request,
        extensions: &mut task_local_extensions::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        req.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token.lock().await.as_ref().unwrap())
                .parse()
                .unwrap(),
        );
        let cloned_req = req
            .try_clone()
            .expect("The request that can not be cloned is not supported.");
        let res = next.clone().run(cloned_req, extensions).await?;
        let status = res.status();
        if !(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN) {
            return Ok(res);
        }

        warn!("The request to Ceph API Server was rejected due to Authorization.");
        info!("Refreshing the authentication token...");
        self.refresh_token().await.unwrap();
        req.headers_mut().insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token.lock().await.as_ref().unwrap())
                .parse()
                .unwrap(),
        );
        let res = next.run(req, extensions).await?;
        let status = res.status();
        if !(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN) {
            return Ok(res);
        }

        panic!("Failed to refresh the authentication token.");
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginSuccess {
    token: String,
    username: String,
}
