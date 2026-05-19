use crate::device::{CameraDevice, Device, UartDeviceConfig};
use tracing::info;

pub fn register_camera(args: &[String], uart: UartDeviceConfig) -> Device {
    info!("Registering camera with args {:?}", args);
    Device::Camera(CameraDevice::from_args(args, uart).expect("invalid camera config"))
}
