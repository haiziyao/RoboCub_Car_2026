mod camera;
mod color;
mod config;
mod cross;
mod qr;
mod response;

#[cfg(test)]
mod tests;

pub use color::run_color_detect;
pub use config::{CameraDevice, ColorDetectConfig, CrossDetectConfig, QrDetectConfig};
pub use cross::run_cross_detect;
pub use qr::run_qr_detect;
pub use response::{
    ResponseDeviceConfig, TaskLightKind, UartDeviceConfig, begin_light_session, send_response_line,
};
