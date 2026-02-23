use crate::config::device_config::DeviceConfig;
use anyhow::Result; 

/// 这个文件其实有点尴尬
/// 主要是读取所有的toml配置文件
pub fn get_config() -> Result<DeviceConfig> {
    let filepath = "config/param.toml";
    DeviceConfig::from_file(filepath)
}         