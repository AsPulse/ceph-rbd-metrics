use axum::http::StatusCode;
use axum::response::Html;

pub async fn landing() -> (StatusCode, Html<&'static str>) {
    (StatusCode::OK, Html(include_str!("./index.html")))
}
