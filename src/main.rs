use car_cv::utils::device_config_util::get_config;
use car_cv::device::color_detect::color_detect_work;
use anyhow::{Result};


fn main() -> Result<()> {
    let my_config = get_config()?; 
    color_detect_work::work(my_config.color_camera_config, true)?;
   Ok(())
}




#[test]
fn test_color_detect() -> Result<()> {
    let my_config = get_config()?; 
    color_detect_work::work(my_config.color_camera_config, true)?;
   Ok(())
}