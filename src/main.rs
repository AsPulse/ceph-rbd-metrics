use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;

use self::ceph::CephRestfulClient;
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::{Extension, Router};
use dotenv::dotenv;
use log::info;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tokio::net::TcpListener;
mod ceph;

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

async fn metrics_handler(Extension(client): Extension<Arc<CephClient>>) -> (StatusCode, String) {
    info!("Requesting metrics...");
    let mut registry = <Registry>::default();
    let up = Family::<(), Gauge>::default();
    registry.register(
        "ceph_rbd_up",
        "Value is 1 if the ceph mgr providing api endpoint is up, 0 otherwise",
        up.clone(),
    );
    let Ok(images) = client.client.list_images().await else {
        up.get_or_create(&()).set(0);
        let mut buffer = String::new();
        if encode(&mut buffer, &registry).is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
        }
        return (StatusCode::INTERNAL_SERVER_ERROR, buffer);
    };
    up.get_or_create(&()).set(1);
    let mut buffer = String::new();
    if encode(&mut buffer, &registry).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
    }
    (StatusCode::OK, buffer)
}

async fn landing() -> (StatusCode, Html<&'static str>) {
    (StatusCode::OK, Html(include_str!("./index.html")))
}

fn get_port_in_env() -> Option<u16> {
    env::var("PORT").ok()?.parse::<u16>().ok()
}
