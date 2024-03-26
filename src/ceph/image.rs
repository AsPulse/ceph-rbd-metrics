use serde::Deserialize;

use super::{CephRestfulClient, CephRestfulClientError};

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct ImageInfo {
    size: u64,
    obj_size: u64,
    num_objs: u64,
    name: String,
    id: String,
    pool_name: String,
    total_disk_usage: u64,
    disk_usage: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct ImagesInPoolInfo {
    pool_name: String,
    value: Vec<ImageInfo>,
}

impl CephRestfulClient {
    pub async fn list_images(&self) -> Result<Vec<ImageInfo>, CephRestfulClientError> {
        let response = self
            .client
            .get(self.endpoint("/api/block/image")?)
            .send()
            .await?;
        let images: Vec<ImagesInPoolInfo> = response.json().await?;
        Ok(images.into_iter().flat_map(|f| f.value).collect())
    }
}
