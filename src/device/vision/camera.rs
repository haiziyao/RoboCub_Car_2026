use anyhow::{Context, Result};
use opencv::videoio;

use super::config::{ColorDetectConfig, QrDetectConfig};

pub fn register_color_camera(config: &ColorDetectConfig) -> Result<videoio::VideoCapture> {
    open_camera(&config.path, "color")
}

pub fn register_qr_camera(config: &QrDetectConfig) -> Result<videoio::VideoCapture> {
    open_camera(&config.path, "qr")
}

fn open_camera(path: &str, name: &str) -> Result<videoio::VideoCapture> {
    videoio::VideoCapture::from_file(path, videoio::CAP_V4L2)
        .with_context(|| format!("failed to open {name} camera {path}"))
}
