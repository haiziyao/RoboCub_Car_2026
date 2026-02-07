use anyhow::{bail, Result};
use opencv::{
    core,
    highgui,
    imgcodecs,
    imgproc,
    prelude::*,
    types,
    videoio,
};

fn main() -> Result<()> {
    // 1) 打开摄像头（也可以改成 from_file("/dev/video2", CAP_V4L2)）
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        bail!("camera not opened");
    }

    highgui::named_window("show", highgui::WINDOW_NORMAL)?;

    let mut frame = Mat::default();
    let mut mode: i32 = 1;

    loop {
        cam.read(&mut frame)?;
        if frame.empty() {
            continue;
        }

        let out = match mode {
            1 => demo_bgr_inrange(&frame)?,        // BGR 直接筛选（近似 RGB 筛选）
            2 => demo_hsv_inrange(&frame)?,        // HSV 筛选
            3 => demo_threshold(&frame)?,          // 二值化（gray->binary）
            4 => demo_morph_open(&frame)?,         // 开运算
            5 => demo_morph_close(&frame)?,        // 闭运算
            6 => demo_blur_box(&frame)?,           // 均值 blur
            7 => demo_blur_gaussian(&frame)?,      // GaussianBlur
            8 => demo_edges_canny(&frame)?,        // Canny 边缘
            9 => demo_contours_and_draw(&frame)?,  // 轮廓+画框
            _ => frame.clone(),
        };

        // 叠字显示模式
        let mut show = out;
        imgproc::put_text(
            &mut show,
            &format!("mode: {} (press 1-9, q quit)", mode),
            core::Point::new(20, 30),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.8,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            false,
        )?;

        highgui::imshow("show", &show)?;
        let key = highgui::wait_key(1)?;
        if key == 113 { // q
            break;
        }
        // '1'..'9'
        if key >= 49 && key <= 57 {
            mode = key - 48;
        }
    }

    Ok(())
}

/* =========================
   0) 常用工具函数
   ========================= */

fn to_gray(bgr: &Mat) -> Result<Mat> {
    let mut gray = Mat::default();
    imgproc::cvt_color(bgr, &mut gray, imgproc::COLOR_BGR2GRAY, 0)?;
    Ok(gray)
}

fn kernel_ellipse(ksize: i32) -> Result<Mat> {
    Ok(imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        core::Size::new(ksize, ksize),
        core::Point::new(-1, -1),
    )?)
}

/* =========================
   1) BGR（近似 RGB）inRange 筛选
   说明：相机帧是 BGR
   ========================= */
fn demo_bgr_inrange(bgr: &Mat) -> Result<Mat> {
    // 例：筛“偏红”的区域（只是示例阈值，你自己调）
    // BGR: (B,G,R)
    let lower = core::Scalar::new(0.0, 0.0, 150.0, 0.0);
    let upper = core::Scalar::new(80.0, 80.0, 255.0, 0.0);

    let mut mask = Mat::default();
    core::in_range(bgr, &lower, &upper, &mut mask)?;

    // 为了显示好看：mask 转成 3 通道叠回去
    let mut out = Mat::default();
    imgproc::cvt_color(&mask, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   2) HSV inRange 筛选
   ========================= */
fn demo_hsv_inrange(bgr: &Mat) -> Result<Mat> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(bgr, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    // 例：筛蓝色（示例阈值）
    let lower = core::Scalar::new(100.0, 80.0, 80.0, 0.0);
    let upper = core::Scalar::new(140.0, 255.0, 255.0, 0.0);

    let mut mask = Mat::default();
    core::in_range(&hsv, &lower, &upper, &mut mask)?;

    let mut out = Mat::default();
    imgproc::cvt_color(&mask, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   3) 二值化 threshold
   ========================= */
fn demo_threshold(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;

    let mut bin = Mat::default();
    imgproc::threshold(&gray, &mut bin, 127.0, 255.0, imgproc::THRESH_BINARY)?;

    let mut out = Mat::default();
    imgproc::cvt_color(&bin, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   4) 开运算（去小白噪点）
   ========================= */
fn demo_morph_open(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;
    let mut bin = Mat::default();
    imgproc::threshold(&gray, &mut bin, 127.0, 255.0, imgproc::THRESH_BINARY)?;

    let k = kernel_ellipse(5)?;
    let mut out_bin = Mat::default();
    imgproc::morphology_ex(
        &bin,
        &mut out_bin,
        imgproc::MORPH_OPEN,
        &k,
        core::Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;

    let mut out = Mat::default();
    imgproc::cvt_color(&out_bin, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   5) 闭运算（补小黑洞）
   ========================= */
fn demo_morph_close(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;
    let mut bin = Mat::default();
    imgproc::threshold(&gray, &mut bin, 127.0, 255.0, imgproc::THRESH_BINARY)?;

    let k = kernel_ellipse(7)?;
    let mut out_bin = Mat::default();
    imgproc::morphology_ex(
        &bin,
        &mut out_bin,
        imgproc::MORPH_CLOSE,
        &k,
        core::Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;

    let mut out = Mat::default();
    imgproc::cvt_color(&out_bin, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   6) blur（均值模糊 / Box Filter）
   ========================= */
fn demo_blur_box(bgr: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::blur(
        bgr,
        &mut out,
        core::Size::new(9, 9),
        core::Point::new(-1, -1),
        core::BORDER_DEFAULT,
    )?;
    Ok(out)
}

/* =========================
   7) GaussianBlur（高斯模糊）
   ========================= */
fn demo_blur_gaussian(bgr: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::gaussian_blur(
        bgr,
        &mut out,
        core::Size::new(9, 9),
        2.0,
        2.0,
        core::BORDER_DEFAULT,
    )?;
    Ok(out)
}

/* =========================
   8) Canny 边缘
   ========================= */
fn demo_edges_canny(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;

    let mut edges = Mat::default();
    imgproc::canny(&gray, &mut edges, 80.0, 160.0, 3, false)?;

    let mut out = Mat::default();
    imgproc::cvt_color(&edges, &mut out, imgproc::COLOR_GRAY2BGR, 0)?;
    Ok(out)
}

/* =========================
   9) 轮廓检测 + 画框/画轮廓
   ========================= */
fn demo_contours_and_draw(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;
    let mut bin = Mat::default();
    imgproc::threshold(&gray, &mut bin, 127.0, 255.0, imgproc::THRESH_BINARY)?;

    let mut contours = types::VectorOfVectorOfPoint::new();
    imgproc::find_contours(
        &bin,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    )?;

    let mut out = bgr.clone();

    for i in 0..contours.len() {
        let c = contours.get(i)?;
        let area = imgproc::contour_area(&c, false)?;
        if area < 500.0 {
            continue;
        }

        // 外接矩形
        let rect = imgproc::bounding_rect(&c)?;
        imgproc::rectangle(
            &mut out,
            rect,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            0,
        )?;

        // 画轮廓
        imgproc::draw_contours(
            &mut out,
            &contours,
            i as i32,
            core::Scalar::new(0.0, 0.0, 255.0, 0.0),
            2,
            imgproc::LINE_8,
            &core::no_array(),
            i32::MAX,
            core::Point::new(0, 0),
        )?;
    }

    Ok(out)
}

/* =========================
   额外常用操作（你随时能抄走）
   ========================= */

// 腐蚀/膨胀（erode/dilate）
#[allow(dead_code)]
fn erode_then_dilate(mask: &Mat) -> Result<Mat> {
    let k = kernel_ellipse(5)?;
    let mut tmp = Mat::default();
    imgproc::erode(mask, &mut tmp, &k, core::Point::new(-1, -1), 1, core::BORDER_CONSTANT, core::Scalar::all(0.0))?;
    let mut out = Mat::default();
    imgproc::dilate(&tmp, &mut out, &k, core::Point::new(-1, -1), 1, core::BORDER_CONSTANT, core::Scalar::all(0.0))?;
    Ok(out)
}

// MedianBlur（椒盐噪声很好用）
#[allow(dead_code)]
fn demo_blur_median(bgr: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::median_blur(bgr, &mut out, 7)?;
    Ok(out)
}

// BilateralFilter（保边去噪）
#[allow(dead_code)]
fn demo_blur_bilateral(bgr: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::bilateral_filter(bgr, &mut out, 9, 75.0, 75.0, core::BORDER_DEFAULT)?;
    Ok(out)
}

// resize
#[allow(dead_code)]
fn demo_resize(bgr: &Mat) -> Result<Mat> {
    let mut out = Mat::default();
    imgproc::resize(
        bgr,
        &mut out,
        core::Size::new(640, 480),
        0.0,
        0.0,
        imgproc::INTER_LINEAR,
    )?;
    Ok(out)
}

// ROI 裁剪（只看中心区域）
#[allow(dead_code)]
fn demo_roi(bgr: &Mat) -> Result<Mat> {
    let x = bgr.cols() / 4;
    let y = bgr.rows() / 4;
    let w = bgr.cols() / 2;
    let h = bgr.rows() / 2;
    let roi = core::Rect::new(x, y, w, h);
    Ok(Mat::roi(bgr, roi)?.clone())
}

// 霍夫直线（在边缘图上）
#[allow(dead_code)]
fn demo_hough_lines(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;
    let mut edges = Mat::default();
    imgproc::canny(&gray, &mut edges, 80.0, 160.0, 3, false)?;

    let mut lines = types::VectorOfVec2f::new();
    imgproc::hough_lines(&edges, &mut lines, 1.0, std::f64::consts::PI / 180.0, 150, 0.0, 0.0)?;

    let mut out = bgr.clone();
    for l in lines.iter().take(20) {
        let rho = l[0] as f64;
        let theta = l[1] as f64;
        let a = theta.cos();
        let b = theta.sin();
        let x0 = a * rho;
        let y0 = b * rho;
        let pt1 = core::Point::new((x0 + 1000.0 * (-b)) as i32, (y0 + 1000.0 * a) as i32);
        let pt2 = core::Point::new((x0 - 1000.0 * (-b)) as i32, (y0 - 1000.0 * a) as i32);
        imgproc::line(&mut out, pt1, pt2, core::Scalar::new(0.0, 255.0, 0.0, 0.0), 2, imgproc::LINE_8, 0)?;
    }
    Ok(out)
}

// 霍夫圆（建议在灰度+模糊后）
#[allow(dead_code)]
fn demo_hough_circles(bgr: &Mat) -> Result<Mat> {
    let gray = to_gray(bgr)?;
    let mut blur = Mat::default();
    imgproc::gaussian_blur(&gray, &mut blur, core::Size::new(9, 9), 2.0, 2.0, core::BORDER_DEFAULT)?;

    let mut circles = types::VectorOfVec3f::new();
    imgproc::hough_circles(
        &blur,
        &mut circles,
        imgproc::HOUGH_GRADIENT,
        1.2,
        40.0,
        100.0,
        30.0,
        10,
        0,
    )?;

    let mut out = bgr.clone();
    for c in circles.iter() {
        let x = c[0].round() as i32;
        let y = c[1].round() as i32;
        let r = c[2].round() as i32;
        imgproc::circle(&mut out, core::Point::new(x, y), r, core::Scalar::new(0.0, 255.0, 0.0, 0.0), 2, imgproc::LINE_8, 0)?;
    }
    Ok(out)
}
