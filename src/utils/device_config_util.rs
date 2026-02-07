use crate::config::device_config::DeviceConfig;
use anyhow::Result; 


pub fn get_config() -> Result<DeviceConfig> {
    let filepath = "config/param.toml";
    DeviceConfig::from_file(filepath)
} 