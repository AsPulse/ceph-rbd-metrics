pub struct TracingMiddleware;
use log::{info, trace};
use reqwest::Response;
use reqwest_middleware::Middleware;

#[async_trait::async_trait]
impl Middleware for TracingMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut task_local_extensions::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        trace!("{:?}", &req);
        let method = req.method().to_string();
        let url = req.url().to_string();
        let res = next.run(req, extensions).await;
        trace!("{:?}", &res);
        info!(
            "{} {} - {}",
            method,
            url,
            res.as_ref()
                .map(|r| r.status().to_string())
                .unwrap_or("(error)".to_string())
        );
        return res;
    }
}
