use std::sync::Arc;

use log::{info, warn};
use reqwest::header::{HeaderValue, AUTHORIZATION};
use reqwest::{header, Response, StatusCode};
use reqwest_middleware::{ClientBuilder, Middleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use super::tracing::TracingMiddleware;
use super::{CephRestfulClientAccess, ACCEPT_V1, USER_AGENT};

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
        let _ = auth.refresh_token().await;
        auth
    }

    async fn refresh_token(&self) -> reqwest_middleware::Result<()> {
        let token = self.fetch_token().await;
        if token.is_err() {
            warn!("Failed to refresh the authentication token.");
            self.token.lock().await.take();
        }
        token
    }

    async fn fetch_token(&self) -> reqwest_middleware::Result<()> {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = ClientBuilder::new(
            reqwest::Client::builder()
                .default_headers(
                    [
                        (header::USER_AGENT, HeaderValue::from_static(USER_AGENT)),
                        (header::ACCEPT, HeaderValue::from_static(ACCEPT_V1)),
                    ]
                    .into_iter()
                    .collect(),
                )
                .build()
                .unwrap(),
        )
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .with(TracingMiddleware)
        .build();

        let body = json!({
            "username": self.access.username,
            "password": self.access.password,
        });

        let response = client
            .post(self.access.host.join("/api/auth").unwrap())
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
                panic!(
                    "Failed to login, given credential is wrong.\n {:?}.",
                    response.text().await
                );
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                warn!("Failed to login, the server is not available.");
                Ok(())
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
        if let Some(token) = self.token.lock().await.as_ref() {
            req.headers_mut()
                .insert(AUTHORIZATION, format!("Bearer {}", token).parse().unwrap());
            let cloned_req = req
                .try_clone()
                .expect("The request that can not be cloned is not supported.");
            let res = next.clone().run(cloned_req, extensions).await?;
            let status = res.status();
            if !(status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN) {
                return Ok(res);
            }

            warn!("The request to Ceph API Server was rejected due to Authorization.");
        } else {
            warn!("The authentication token is not present.");
        }
        info!("Refreshing the authentication token...");
        self.refresh_token().await?;
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
