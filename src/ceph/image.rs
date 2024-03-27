use log::info;
use prometheus_client::encoding::EncodeLabelSet;
use serde::Deserialize;

use super::{CephRestfulClient, CephRestfulClientError};

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct ImageInfo {
    pub size: u64,
    pub obj_size: u64,
    pub num_objs: u64,
    pub name: String,
    pub id: String,
    pub pool_name: String,
    pub total_disk_usage: u64,
    pub disk_usage: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct ImagesInPoolInfo {
    pool_name: String,
    value: Vec<ImageInfo>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ImageMetadata {
    pub image_name: String,
    pub image_id: String,
    pub pool_name: String,
}

impl From<ImageInfo> for ImageMetadata {
    fn from(info: ImageInfo) -> Self {
        Self {
            image_name: info.name,
            image_id: info.id,
            pool_name: info.pool_name,
        }
    }
}
impl CephRestfulClient {
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>, CephRestfulClientError> {
        // Firstly fetch the images with 1 limit to get the total count.
        let mut ep = self.endpoint("/api/block/image")?;
        ep.query_pairs_mut().append_pair("limit", "1");
        let response = self.client.get(ep).send().await?;
        let count = response
            .headers()
            .get("X-Total-Count")
            .unwrap()
            .to_str()
            .unwrap();
        info!("Fetching {} images...", count);

        // Fetch all images with the total count.
        let mut ep = self.endpoint("/api/block/image")?;
        ep.query_pairs_mut().append_pair("limit", count);
        let response = self.client.get(ep).send().await?;
        let images: Vec<ImagesInPoolInfo> = response.json().await?;
        Ok(images.into_iter().flat_map(|f| f.value).collect())
    }
}
