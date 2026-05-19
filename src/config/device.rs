use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct DeviceParamConfig {
    pub uart_config: UartParam,
    pub device_config_list: Vec<DeviceParam>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct UartParam {
    pub serial: String,
    pub baud: u32,
    pub data_bit: u8,
    pub stop_bit: u8,
    pub parity_bit: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DeviceParam {
    pub device_id: String,
    pub kind: String,
    pub args: Vec<String>,
}

impl DeviceParamConfig {}

impl DeviceParam {}
