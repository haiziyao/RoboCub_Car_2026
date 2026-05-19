use anyhow::Result;

use super::config::CrossDetectConfig;

pub fn run_cross_detect(config: &CrossDetectConfig) -> Result<String> {
    let _path = &config.path;
    Ok("0".to_string())
}
