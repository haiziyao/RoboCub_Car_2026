use crate::config::{DeviceParam, DeviceParamConfig};
use crate::device::UartDeviceConfig;
use crate::device::usual::*;
use crate::device::{Device, DeviceMap};

pub fn register_device(config: DeviceParamConfig) -> DeviceMap {
    let DeviceParamConfig {
        uart_config,
        device_config_list,
    } = config;
    let runtime_uart = UartDeviceConfig::from_param(&uart_config)
        .expect("invalid device_param_config.uart_config");

    let mut map = DeviceMap::new();
    device_config_list.iter().for_each(|device_config| {
        let DeviceParam {
            device_id,
            kind,
            args,
        } = device_config;
        map.add(device_id, device_factory(kind, args, runtime_uart.clone()));
    });
    map
}

fn device_factory(kind: &str, args: &[String], uart: UartDeviceConfig) -> Device {
    match kind {
        "Camera" => register_camera(args, uart),
        _ => panic!("unknown device kind `{kind}`"),
    }
}
