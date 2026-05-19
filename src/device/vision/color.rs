use anyhow::Result;
use opencv::{
    core::{self, Mat, Scalar},
    highgui, imgproc,
    prelude::*,
};

use crate::utils::cv_util::{hsv_inrange, hsv_scalar_factory, roi_circle_mask};

use super::camera::register_color_camera;
use super::config::ColorDetectConfig;

pub fn run_color_detect(config: &ColorDetectConfig) -> Result<String> {
    let mut cam = register_color_camera(config)?;
    let mut best_color = String::new();
    let mut count = 0;
    let stable_count = config.loop_count;

    loop {
        let mut frame = core::Mat::default();
        cam.read(&mut frame)?;
        if frame.empty() {
            continue;
        }

        let (_roi, circle_mask) = roi_circle_mask(&frame, config.radius_ratio)?;
        let (color_name, ratio) = detect_color_in_circle_mask(&frame, &circle_mask, config)?;

        if config.debug_model {
            draw_debug_info(&mut frame, &color_name, ratio, config.radius_ratio)?;
            let key = highgui::wait_key(1)?;
            if key == 113 || key == 27 {
                break;
            }
            continue;
        }

        if count == 0 {
            best_color = color_name;
            count = 1;
        } else if best_color == color_name {
            count += 1;
        } else {
            best_color.clear();
            count = 0;
        }

        if count >= stable_count {
            return Ok(best_color);
        }
    }

    Ok(String::new())
}

fn color_ratio_in_circle_mask(
    frame_bgr: &Mat,
    circle_mask: &Mat,
    hsv_arr: [i32; 6],
) -> Result<f64> {
    let (lower, upper) = hsv_scalar_factory(hsv_arr)?;
    let color_mask = hsv_inrange(frame_bgr, &lower, &upper)?;

    let mut inside = Mat::default();
    core::bitwise_and(&color_mask, &color_mask, &mut inside, circle_mask)?;

    let hit = core::count_non_zero(&inside)? as f64;
    let total = core::count_non_zero(circle_mask)? as f64;

    Ok(if total > 0.0 { hit / total } else { 0.0 })
}

fn detect_color_in_circle_mask(
    frame_bgr: &Mat,
    circle_mask: &Mat,
    config: &ColorDetectConfig,
) -> Result<(String, f64)> {
    let mut best_name = "unknown".to_string();
    let mut best_ratio = 0.0_f64;

    for color in &config.color_ranges {
        let ratio = color_ratio_in_circle_mask(frame_bgr, circle_mask, color.hsv)?;

        if ratio > best_ratio {
            best_ratio = ratio;
            best_name = color.name.clone();
        }
    }

    if best_ratio >= config.detect_area_access_rate {
        Ok((best_name, best_ratio))
    } else {
        Ok(("unknown".to_string(), best_ratio))
    }
}

fn draw_label(frame: &mut Mat, text: &str, x: i32, y: i32) -> Result<()> {
    imgproc::put_text(
        frame,
        text,
        core::Point::new(x, y),
        imgproc::FONT_HERSHEY_SIMPLEX,
        0.8,
        Scalar::new(255.0, 255.0, 255.0, 0.0),
        2,
        imgproc::LINE_AA,
        false,
    )?;
    Ok(())
}

fn draw_debug_info(frame: &mut Mat, color_name: &str, ratio: f64, radius_ratio: f64) -> Result<()> {
    let size = frame.size()?;
    let w = size.width;
    let h = size.height;
    let cx = w / 2;
    let cy = h / 2;
    let r = ((w.min(h) as f64) * radius_ratio) as i32;

    imgproc::circle(
        frame,
        core::Point::new(cx, cy),
        r,
        core::Scalar::new(0.0, 255.0, 0.0, 0.0),
        2,
        imgproc::LINE_8,
        0,
    )?;

    let label = format!("color: {}  ratio: {:.2}", color_name, ratio);
    draw_label(frame, &label, 10, 30)?;
    highgui::imshow("color_detect", frame)?;
    Ok(())
}
