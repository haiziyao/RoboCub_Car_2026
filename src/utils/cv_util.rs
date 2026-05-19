#![allow(dead_code)]

use anyhow::Result;
use opencv::{core, imgproc, prelude::*};

#[derive(Copy, Clone, Debug)]
pub enum KernelShape {
    Rect,
    Ellipse,
    Cross,
}

fn kernel_factory(ksize: i32, shape: KernelShape) -> Result<Mat> {
    let shape = match shape {
        KernelShape::Rect => imgproc::MORPH_RECT,
        KernelShape::Ellipse => imgproc::MORPH_ELLIPSE,
        KernelShape::Cross => imgproc::MORPH_CROSS,
    };

    Ok(imgproc::get_structuring_element(
        shape,
        core::Size::new(ksize, ksize),
        core::Point::new(-1, -1),
    )?)
}

pub fn to_bgr_out(original: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::cvt_color(original, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

pub fn bgr_to_gray(bgr: &Mat) -> Result<Mat> {
    let mut gray = Mat::default();
    imgproc::cvt_color(bgr, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
    Ok(gray)
}

pub fn bgr_inrange(bgr: &Mat, lower: &core::Scalar, upper: &core::Scalar) -> Result<Mat> {
    let mut mask = Mat::default();
    core::in_range(bgr, lower, upper, &mut mask)?;
    Ok(mask)
}

pub fn hsv_scalar_factory(hsv: [i32; 6]) -> Result<(core::Scalar, core::Scalar)> {
    let lower = core::Scalar::new(hsv[0] as f64, hsv[2] as f64, hsv[4] as f64, 0.0);
    let upper = core::Scalar::new(hsv[1] as f64, hsv[3] as f64, hsv[5] as f64, 0.0);
    Ok((lower, upper))
}

pub fn hsv_inrange(bgr: &Mat, lower: &core::Scalar, upper: &core::Scalar) -> Result<Mat> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(bgr, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let mut mask = Mat::default();
    core::in_range(&hsv, lower, upper, &mut mask)?;
    Ok(mask)
}

pub fn threshold(bgr: &Mat, thresh: f64, maxval: f64) -> Result<Mat> {
    let gray = bgr_to_gray(bgr)?;

    let mut bin = Mat::default();
    imgproc::threshold(&gray, &mut bin, thresh, maxval, imgproc::THRESH_BINARY)?;
    Ok(bin)
}

pub fn morph_open(bin: &Mat, kernel: &Mat) -> Result<Mat> {
    let mut out_bin = Mat::default();
    imgproc::morphology_ex(
        bin,
        &mut out_bin,
        imgproc::MORPH_OPEN,
        kernel,
        core::Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;
    Ok(out_bin)
}

pub fn morph_close(bin: &Mat, kernel: &Mat) -> Result<Mat> {
    let mut out_bin = Mat::default();
    imgproc::morphology_ex(
        bin,
        &mut out_bin,
        imgproc::MORPH_CLOSE,
        kernel,
        core::Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;
    Ok(out_bin)
}

pub fn bgr_blur_box(bgr: &Mat, ksize: i32) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::blur(
        bgr,
        &mut out,
        core::Size::new(ksize, ksize),
        core::Point::new(-1, -1),
        core::BORDER_DEFAULT,
    )?;
    Ok(out)
}

pub fn bgr_blur_gaussian(bgr: &Mat, ksize: i32) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::gaussian_blur(
        bgr,
        &mut out,
        core::Size::new(ksize, ksize),
        0.0,
        0.0,
        core::BORDER_DEFAULT,
    )?;
    Ok(out)
}

pub fn roi_circle_mask(frame_bgr: &Mat, radius_ratio: f64) -> Result<(Mat, Mat)> {
    let size = frame_bgr.size()?;
    let w = size.width;
    let h = size.height;

    let cx = w / 2;
    let cy = h / 2;
    let r = ((w.min(h) as f64) * radius_ratio) as i32;
    let mut mask = Mat::zeros(h, w, core::CV_8UC1)?.to_mat()?;
    imgproc::circle(
        &mut mask,
        core::Point::new(cx, cy),
        r,
        core::Scalar::all(255.0),
        -1,
        imgproc::LINE_8,
        0,
    )?;

    let mut roi = Mat::default();
    core::bitwise_and(frame_bgr, frame_bgr, &mut roi, &mask)?;
    Ok((roi, mask))
}
