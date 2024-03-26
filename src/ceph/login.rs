use log::info;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use super::{CephRestfulClient, CephRestfulClientError};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginSuccess {
    token: String,
    username: String,
}

impl CephRestfulClient {
    pub async fn login(&mut self) -> Result<(), CephRestfulClientError> {
        let body = json!({
            "username": self.access.username,
            "password": self.access.password,
        });

        let response = self
            .client
            .post(self.endpoint("/api/auth")?)
            .json(&body)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token = response.json::<LoginSuccess>().await?;
                info!("Logged in as {}", token.username);
                self.token = Some(token.token);
                Ok(())
            }
            StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED => {
                panic!("Failed to login.\n {:?}.", response.text().await);
            }
            _ => panic!("Unexpected response: {:?}", response),
        }
    }
}
