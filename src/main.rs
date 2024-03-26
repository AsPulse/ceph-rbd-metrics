use std::env;

use dotenv::dotenv;
use url::Url;

use self::ceph::{CephRestfulClient, CephRestfulClientAccess};

mod ceph;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let access = CephRestfulClientAccess {
        host: Url::parse(&env::var("CEPH_API_ENDPOINT").expect("CEPH_API_ENDPOINT is not set"))
            .expect("CEPH_API_ENDPOINT is not a valid URL"),
        username: env::var("CEPH_API_USERNAME").expect("CEPH_API_USERNAME is not set"),
        password: env::var("CEPH_API_PASSWORD").expect("CEPH_API_PASSWORD is not set"),
    };

    let mut client = CephRestfulClient::new(access);
    client.login().await?;
    Ok(())
}
