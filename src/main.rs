use self::ceph::CephRestfulClient;
use dotenv::dotenv;
mod ceph;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let client = CephRestfulClient::from_env().await;
    Ok(())
}
