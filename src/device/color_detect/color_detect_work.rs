//! 核心算法层：仅包含圆形 ROI 与 HSV 颜色判定逻辑，不参与线程/通道/界面。
use crate::{config::device_config::ColorCameraConfig, device::camera};
use crate::utils::cv_util::{roi_circle_mask,hsv_inrange,hsv_scalar_factory};
use anyhow::{Ok, Result};
use opencv::{
    core::{self, Mat, Scalar},
    highgui,
    imgproc,
    prelude::*,
};



 

/// 在摄像头帧中基于圆形 ROI 计算主色（算法层入口，不涉及线程与 UI 路由）。
pub fn work(config: ColorCameraConfig) -> Result<String> {
    let mut cam = camera::register_color_camera(config.clone())?;

    let mut best_color: String = String::new();
   
    let mut count:i32 = 0;

    loop {
        let mut frame = core::Mat::default();
        cam.read(&mut frame)?;
        if frame.empty() { continue; }

        // 基于短边比例生成圆形 ROI 掩码，后续仅统计圆内像素。
        let (_roi, circle_mask) = roi_circle_mask(&frame, config.radius_ratio)?;
        // 在圆形 ROI 内统计各颜色占比，选取占比最高者。
        let (color_name, ratio) = detect_color_in_circle_mask(&frame, &circle_mask, &config)?;

        if config.debug_model {
            // 调试模式：叠加 ROI 与检测信息，便于标定阈值。
            draw_debug_info(&mut frame, &color_name, ratio, config.radius_ratio)?;

            let key = highgui::wait_key(1)?;
            if key == 113 || key == 27 {
                break; // q / esc 退出
            }
        }
        if !config.debug_model{
            // 量产模式：做连续帧稳定性过滤，避免偶发误检。
            if count == 0 {
                best_color = color_name.clone();   
                count += 1;
            } else {
                if best_color == color_name {
                    count += 1;
                } else {
                    best_color.clear();   
                    count = 0;
                }
            }
            if count > 10 {
                return Ok(best_color);
            }
        }
    }

    Ok(String::new())
}


 
/// 计算圆形 ROI 内指定 HSV 区间的像素占比。
pub fn color_ratio_in_circle_mask(
    frame_bgr: &Mat,circle_mask: &Mat,hsv_arr: [i32; 6]) -> Result<f64> {

    let (lower, upper) = hsv_scalar_factory(hsv_arr)?;

    // HSV 阈值分割：将目标颜色区域转成二值掩码。
    let color_mask = hsv_inrange(frame_bgr, &lower, &upper)?;

    let mut inside = Mat::default();
    // 与圆形 ROI 掩码相交，只保留圆内像素。
    core::bitwise_and(&color_mask, &color_mask, &mut inside, circle_mask)?;

    let hit = core::count_non_zero(&inside)? as f64;
    let total = core::count_non_zero(circle_mask)? as f64;

    // 命中像素 / ROI 像素 = 占比。
    Ok(if total > 0.0 { hit / total } else { 0.0 })
}

/// 在圆形 ROI 内遍历配置颜色，返回占比最高且达阈值的颜色。
pub fn detect_color_in_circle_mask(
    frame_bgr: &Mat,circle_mask: &Mat,config: &ColorCameraConfig,) 
    -> Result<(String, f64)> {


    let mut best_name = "unknown".to_string();
    let mut best_ratio = 0.0_f64;

    // 对配置中的颜色逐个计算 ROI 内占比。
    for c in &config.colors {
        let (name, hsv_arr) = match c.as_str() {
            "red" => ("red", config.hsv_red),
            "blue" => ("blue", config.hsv_blue),
            "green" => ("green", config.hsv_green),
            "black" => ("black", config.hsv_black),
            "white" => ("white", config.hsv_white),
            _ => continue,
        };

        let ratio = color_ratio_in_circle_mask(frame_bgr,circle_mask,hsv_arr)?;

        if ratio > best_ratio {
            best_ratio = ratio;
            best_name = name.to_string();
        }
    }

    // 低于阈值则视为 unknown，避免弱噪声触发。
    if best_ratio >= config.detect_area_access_rate {
        Ok((best_name, best_ratio))
    } else {
        Ok(("unknown".to_string(), best_ratio))
    }
}



// 绘制调试文字，供算法标定使用。
fn draw_label(frame: &mut Mat, text: &str, x: i32, y: i32) -> Result<()> {
    imgproc::put_text(
        frame,
        text,
        core::Point::new(x, y),
        imgproc::FONT_HERSHEY_SIMPLEX,
        0.8,
        Scalar::new(255.0, 255.0, 255.0, 0.0), // 白字
        2,
        imgproc::LINE_AA,
        false,
    )?;
    Ok(())
}

/// 在帧上叠加 ROI 圆圈与识别结果，便于 Web/UI 观察算法决策。
pub fn draw_debug_info(frame: &mut core::Mat, color_name: &str, ratio: f64, radius_ratio: f64) -> Result<()> {
    let size = frame.size()?;
    let w = size.width;
    let h = size.height;
    let cx = w / 2;
    let cy = h / 2;
    let r = ((w.min(h) as f64) * radius_ratio) as i32;

    // 画出圆形 ROI，展示算法的有效区域。
    imgproc::circle(
        frame,
        core::Point::new(cx, cy),
        r,
        core::Scalar::new(0.0, 255.0, 0.0, 0.0),
        2,
        imgproc::LINE_8,
        0,
    )?;

    // 显示标签（颜色 + 占比）。
    let label = format!("color: {}  ratio: {:.2}", color_name, ratio);
    draw_label(frame, &label, 10, 30)?;

    // 显示图像（如需本地调试可开启）。
   // highgui::imshow("color_detect", frame)?;
    Ok(())
}