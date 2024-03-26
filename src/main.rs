use crate::router::landing::landing;
use crate::router::metrics::metrics_handler;

use self::ceph::CephRestfulClient;
use axum::routing::get;
use axum::{Extension, Router};
use dotenv::dotenv;
use log::info;
use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::net::TcpListener;
mod ceph;
mod router;

pub struct CephClient {
    client: CephRestfulClient,
}
impl CephClient {
    pub async fn new() -> Self {
        Self {
            client: CephRestfulClient::from_env().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(landing))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(Arc::new(CephClient::new().await)));
    let port = get_port_in_env().unwrap_or(3000);
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();
    info!("Server listening on port {}", port);
    axum::serve(listener, app).await?;
    Ok(())
}

fn get_port_in_env() -> Option<u16> {
    env::var("PORT").ok()?.parse::<u16>().ok()
}
