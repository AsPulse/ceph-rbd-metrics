use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::Extension;
use log::{error, info, warn};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;

use crate::ceph::image::ImageMetadata;
use crate::CephClient;

pub async fn metrics_handler(
    Extension(client): Extension<Arc<CephClient>>,
) -> (StatusCode, String) {
    info!("Requesting metrics...");
    let mut registry = <Registry>::with_prefix("ceph_rbd");
    let up = Gauge::<i64, AtomicI64>::default();
    registry.register(
        "up",
        "Value is 1 if the ceph mgr providing api endpoint is up, 0 otherwise",
        up.clone(),
    );
    let Ok(images) = client.client.list_images().await else {
        up.set(0i64);
        let mut buffer = String::new();
        warn!("Failed to communicate with ceph mgr.");
        if encode(&mut buffer, &registry).is_err() {
            error!("Failed to encode metrics. Returning 500...");
            return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
        }
        return (StatusCode::OK, buffer);
    };
    up.set(1i64);
    info!("Metrics successfully fetched from ceph mgr.");

    let size = Family::<ImageMetadata, Gauge>::default();
    let obj_size = Family::<ImageMetadata, Gauge>::default();
    let num_objs = Family::<ImageMetadata, Gauge>::default();
    let total_disk_usage = Family::<ImageMetadata, Gauge>::default();
    let disk_usage = Family::<ImageMetadata, Gauge>::default();

    registry.register_with_unit(
        "image_size",
        "Allocated size of the image",
        prometheus_client::registry::Unit::Bytes,
        size.clone(),
    );
    registry.register_with_unit(
        "image_obj_size",
        "Size of the image object",
        prometheus_client::registry::Unit::Bytes,
        obj_size.clone(),
    );
    registry.register(
        "image_num_objs",
        "Number of objects in the image",
        num_objs.clone(),
    );
    registry.register_with_unit(
        "image_total_disk_usage",
        "Total disk usage of the image",
        prometheus_client::registry::Unit::Bytes,
        total_disk_usage.clone(),
    );
    registry.register_with_unit(
        "image_disk_usage",
        "Disk usage of the image",
        prometheus_client::registry::Unit::Bytes,
        disk_usage.clone(),
    );

    images.iter().for_each(|image| {
        let label: ImageMetadata = image.clone().into();
        size.get_or_create(&label).set(image.size as i64);
        obj_size.get_or_create(&label).set(image.obj_size as i64);
        num_objs.get_or_create(&label).set(image.num_objs as i64);
        total_disk_usage
            .get_or_create(&label)
            .set(image.total_disk_usage as i64);
        disk_usage
            .get_or_create(&label)
            .set(image.disk_usage as i64);
    });

    log::info!("Metrics of {} images collected.", images.len());

    let mut buffer = String::new();
    if encode(&mut buffer, &registry).is_err() {
        error!("Failed to encode metrics. Returning 500...");
        return (StatusCode::INTERNAL_SERVER_ERROR, String::new());
    }
    info!("Metrics successfully encoded.");
    (StatusCode::OK, buffer)
}
