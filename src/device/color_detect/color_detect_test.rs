//!    已经全部弃用
//!    已经全部弃用
//!    已经全部弃用
//!    已经全部弃用
use crate::config::device_config::{ColorCameraConfig};
use anyhow::{Result, bail};
use opencv::{
    core,
    highgui,
    imgproc,
    prelude::*,
    types,
    videoio,
};



// ===== 预留：GPIO 触发（你自己接通信）=====
fn gpio_triggered() -> bool {
    // TODO: 收到 GPIO/上位机消息 -> true
    false
}

fn gpio_send_color(_color: &str) -> Result<()> {
    // TODO: 这里接你的 GPIO/通信：只发送 color 字符串
    println!("检测到颜色 color:{}",_color);
    Ok(())
}


fn hsv_to_scalars(hsv: [i32; 6]) -> (core::Scalar, core::Scalar) {
    let (hmin, hmax) = if hsv[0] <= hsv[1] { (hsv[0], hsv[1]) } else { (hsv[1], hsv[0]) };
    let (smin, smax) = if hsv[2] <= hsv[3] { (hsv[2], hsv[3]) } else { (hsv[3], hsv[2]) };
    let (vmin, vmax) = if hsv[4] <= hsv[5] { (hsv[4], hsv[5]) } else { (hsv[5], hsv[4]) };

    (
        core::Scalar::new(hmin as f64, smin as f64, vmin as f64, 0.0),
        core::Scalar::new(hmax as f64, smax as f64, vmax as f64, 0.0),
    )
}
 

pub fn img_extra_deal(mask: &Mat) -> Result<Mat> {
    let kernel = imgproc::get_structuring_element(
        imgproc::MORPH_ELLIPSE,
        core::Size::new(5, 5),
        core::Point::new(-1, -1),
    )?;

    // out 保存当前结果
    let mut out = Mat::default();

    // 1) 开运算
    imgproc::morphology_ex(
        mask,           
        &mut out,      
        imgproc::MORPH_OPEN,
        &kernel,
        core::Point::new(-1, -1),
        1,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;

    // 2) 闭运算：必须用 tmp
    let mut tmp = Mat::default();
    imgproc::morphology_ex(
        &out,          
        &mut tmp,       
        imgproc::MORPH_CLOSE,
        &kernel,
        core::Point::new(-1, -1),
        2,
        core::BORDER_CONSTANT,
        core::Scalar::all(0.0),
    )?;
    out = tmp; 

    // 3) 可选：Gaussian blur（同样不能 in-place）
    let mut tmp2 = Mat::default();
    imgproc::gaussian_blur(
        &out,
        &mut tmp2,
        core::Size::new(7, 7),
        1.5,
        1.5,
        core::BORDER_DEFAULT,
    )?;
    out = tmp2;

    Ok(out)
}


fn detect_top_circle_color(
    frame_bgr: &Mat,
    items: &[(&'static str, [i32; 6])],
) -> Result<Option<&'static str>> {
    let mut hsv = Mat::default();
    imgproc::cvt_color(frame_bgr, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let screen_area = (frame_bgr.cols() as f64) * (frame_bgr.rows() as f64);
    let max_white_circle_area = 0.15 * screen_area;

    let mut best: Option<(&'static str, f64)> = None; // (name, score_area)

    for (name, hsv_range) in items.iter() {
        let (lower, upper) = hsv_to_scalars(*hsv_range);

        let mut mask0 = Mat::default();
        core::in_range(&hsv, &lower, &upper, &mut mask0)?;

        // 预处理
        let mask = img_extra_deal(&mask0)?;

        // 找轮廓（外轮廓）
        let mut contours = types::VectorOfVectorOfPoint::new();
        imgproc::find_contours(
            &mask,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            core::Point::new(0, 0),
        )?;
        if contours.is_empty() {
            highgui::named_window(name, highgui::WINDOW_NORMAL)?;
            highgui::imshow(name, &mask)?;
            highgui::wait_key(1)?;
            continue;
        }

        // 选最大轮廓
        let mut max_i = 0usize;
        let mut max_area = -1.0f64;
        for i in 0..contours.len() {
            let c = contours.get(i)?;
            let a = imgproc::contour_area(&c, false)?;
            if a > max_area {
                max_area = a;
                max_i = i;
            }
        }
        let cmax = contours.get(max_i)?;

        // boundingRect，取上半部分点（去掉“杆”）
        let rect = imgproc::bounding_rect(&cmax)?;
        let y_cut = rect.y + rect.height / 2;

        let mut top_points = types::VectorOfPoint::new();
        for p in cmax.iter() {
            if p.y < y_cut {
                top_points.push(p);
            }
        }
        if top_points.len() < 10 {
            // 点太少就不算
            highgui::named_window(name, highgui::WINDOW_NORMAL)?;
            highgui::imshow(name, &mask)?;
            highgui::wait_key(1)?;
            continue;
        }

        // 用上半部分拟合外接圆
        let mut center = core::Point2f::default();
        let mut radius: f32 = 0.0;
        imgproc::min_enclosing_circle(&top_points, &mut center, &mut radius)?;

        let r = radius as f64;
        if r <= 1.0 {
            continue;
        }

        // ===== 0.9 填充率：用“圆内白像素 / 圆面积” =====
        // 先画一个圆形mask，统计圆内白像素
        let mut circle_mask = Mat::zeros(mask.rows(), mask.cols(), core::CV_8UC1)?.to_mat()?;
        imgproc::circle(
            &mut circle_mask,
            core::Point::new(center.x.round() as i32, center.y.round() as i32),
            radius.round() as i32,
            core::Scalar::all(255.0),
            -1,
            imgproc::LINE_8,
            0,
        )?;

        let mut inside = Mat::default();
        core::bitwise_and(&mask, &circle_mask, &mut inside, &core::no_array())?;
        let white_in_circle = core::count_non_zero(&inside)? as f64;

        let circle_area = std::f64::consts::PI * r * r;
        let fill = white_in_circle / circle_area;
        if fill < 0.70 {
            // 不满足你的硬条件
            // 仍然展示
            let mut vis = mask.clone();
            imgproc::circle(
                &mut vis,
                core::Point::new(center.x.round() as i32, center.y.round() as i32),
                radius.round() as i32,
                core::Scalar::new(128.0, 128.0, 128.0, 0.0),
                2,
                imgproc::LINE_8,
                0,
            )?;
            imgproc::put_text(
                &mut vis,
                &format!("fill={:.2}", fill),
                core::Point::new(20, 40),
                imgproc::FONT_HERSHEY_SIMPLEX,
                0.8,
                core::Scalar::all(200.0),
                2,
                imgproc::LINE_8,
                false,
            )?;
            highgui::named_window(name, highgui::WINDOW_NORMAL)?;
            highgui::imshow(name, &vis)?;
            highgui::wait_key(1)?;
            continue;
        }

        // 白色兜底规则（面积<15% 且 圆很少：这里用“轮廓只有一个最大块”近似，不额外算数量）
        if *name == "white" && circle_area >= max_white_circle_area {
            continue;
        }

        // 展示：mask + 拟合圆
        let mut vis = mask.clone();
        imgproc::circle(
            &mut vis,
            core::Point::new(center.x.round() as i32, center.y.round() as i32),
            radius.round() as i32,
            core::Scalar::new(128.0, 128.0, 128.0, 0.0),
            2,
            imgproc::LINE_8,
            0,
        )?;
        imgproc::put_text(
            &mut vis,
            &format!("fill={:.2}", fill),
            core::Point::new(20, 40),
            imgproc::FONT_HERSHEY_SIMPLEX,
            0.8,
            core::Scalar::all(200.0),
            2,
            imgproc::LINE_8,
            false,
        )?;
        highgui::named_window(name, highgui::WINDOW_NORMAL)?;
        highgui::imshow(name, &vis)?;
        highgui::wait_key(1)?;

        // 评分：用圆面积或 r 都行，这里用圆面积
        match best {
            None => best = Some((*name, circle_area)),
            Some((_, best_score)) => {
                if circle_area > best_score {
                    best = Some((*name, circle_area));
                }
            }
        }
    }

    Ok(best.map(|(name, _)| name))
}





pub fn start(config: ColorCameraConfig) -> Result<()> {
    highgui::named_window("window", highgui::WINDOW_FULLSCREEN)?;

    let mut cam = videoio::VideoCapture::from_file(&config.color_camera, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        bail!("无法打开相机/视频源：{}", config.color_camera);
    }

    // items：按 config.colors 初始化（顺序就是你的优先级）
    let mut items: Vec<(&'static str, [i32; 6])> = Vec::new();
    for c in &config.colors {
        match c.as_str() {
            "red" => items.push(("red", config.hsv_red)),
            "blue" => items.push(("blue", config.hsv_blue)),
            "green" => items.push(("green", config.hsv_green)),
            "black" => items.push(("black", config.hsv_black)),
            "white" => items.push(("white", config.hsv_white)),
            _ => {}
        }
    }
    if items.is_empty() {
        bail!("config.colors 里没有可用颜色（red/blue/green/white/black）");
    }

    let mut frame = Mat::default();

    loop {
        cam.read(&mut frame)?;
        if frame.empty() {
            break;
        }

        highgui::imshow("window", &frame)?;
        let key = highgui::wait_key(1)?;
        if key == 113 { // q
            break;
        }

        // 没触发就继续显示，不检测
        if !gpio_triggered() {
            continue;
        }

        // 触发：检测一次
        if let Some(color) = detect_top_circle_color(&frame, &mut items)? {
            gpio_send_color(color)?; // TODO: 只发送颜色
        } else {
            gpio_send_color("none")?; // 你要不要发 none 自己定
        }

        // 检测完：啥也不做，继续回到等待（下一轮 loop 又会卡在 gpio_triggered）
    }

    Ok(())
}



 
#[test]   // TODO这里是测试hsv的配置，以便寻求最好的效果
fn test_hsv() -> Result<()> {
    // 读配置
    let config = crate::utils::device_config_util::get_config()?;
    let config = config.color_camera_config;
    let camera_filename = config.color_camera;

    // ===== 窗口 =====
    highgui::named_window("controls", highgui::WINDOW_AUTOSIZE)?;
    highgui::named_window("frame", highgui::WINDOW_NORMAL)?;
    highgui::named_window("mask", highgui::WINDOW_NORMAL)?;
    highgui::named_window("result", highgui::WINDOW_NORMAL)?;

    // ===== 6 个滑动条 =====
    // 初始值你可以按需改
    let mut h_min: i32 = 0;
    let mut s_min: i32 = 0;
    let mut v_min: i32 = 0;

    let mut h_max: i32 = 179;
    let mut s_max: i32 = 255;
    let mut v_max: i32 = 255;

    // 注意：create_trackbar 需要一个“初始变量地址”，后续我们每一帧用 get_trackbar_pos 取最新值
    highgui::create_trackbar("H min", "controls", Some(&mut h_min), 179, None)?;
    highgui::create_trackbar("H max", "controls", Some(&mut h_max), 179, None)?;
    highgui::create_trackbar("S min", "controls", Some(&mut s_min), 255, None)?;
    highgui::create_trackbar("S max", "controls", Some(&mut s_max), 255, None)?;
    highgui::create_trackbar("V min", "controls", Some(&mut v_min), 255, None)?;
    highgui::create_trackbar("V max", "controls", Some(&mut v_max), 255, None)?;


    // ===== 打开视频/相机 =====
    let mut cam = videoio::VideoCapture::from_file(&camera_filename, videoio::CAP_ANY)?;
    if !videoio::VideoCapture::is_opened(&cam)? {
        anyhow::bail!("无法打开视频/相机：{}", camera_filename);
    }

    let mut frame = Mat::default();
    let mut hsv = Mat::default();
    let mut mask = Mat::default();
    let mut result = Mat::default();

    loop {
        cam.read(&mut frame)?;
        if frame.empty() {
            // 文件读完或相机无帧
            break;
        }

         

        // 取最新滑动条值（每帧刷新）
        let h_min = highgui::get_trackbar_pos("H min", "controls")?;
        let h_max = highgui::get_trackbar_pos("H max", "controls")?;
        let s_min = highgui::get_trackbar_pos("S min", "controls")?;
        let s_max = highgui::get_trackbar_pos("S max", "controls")?;
        let v_min = highgui::get_trackbar_pos("V min", "controls")?;
        let v_max = highgui::get_trackbar_pos("V max", "controls")?;

        // 防呆：min/max 反了就交换（不然 inRange 会全黑）
        let (h1, h2) = if h_min <= h_max { (h_min, h_max) } else { (h_max, h_min) };
        let (s1, s2) = if s_min <= s_max { (s_min, s_max) } else { (s_max, s_min) };
        let (v1, v2) = if v_min <= v_max { (v_min, v_max) } else { (v_max, v_min) };

        // BGR -> HSV
        imgproc::cvt_color(&frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

        
        // HSV 阈值分割
        let lower = core::Scalar::new(h1 as f64, s1 as f64, v1 as f64, 0.0);
        let upper = core::Scalar::new(h2 as f64, s2 as f64, v2 as f64, 0.0);
        core::in_range(&hsv, &lower, &upper, &mut mask)?;

        // 可选：简单去噪（你想更干净就打开）
        // let kernel = imgproc::get_structuring_element(imgproc::MORPH_ELLIPSE, core::Size::new(5, 5), core::Point::new(-1, -1))?;
        // imgproc::morphology_ex(&mask, &mut mask, imgproc::MORPH_OPEN, &kernel, core::Point::new(-1,-1), 1, core::BORDER_CONSTANT, core::Scalar::all(0.0))?;

         
        // TODO选择是否使用额外的图像操作: 膨胀，腐蚀，高斯模糊
        let mask = img_extra_deal(&mask)?;   // ✅ 返回一个新的 Mat，直接用同名变量接住

        core::bitwise_and(&frame, &frame, &mut result, &mask)?;

        highgui::imshow("frame", &frame)?;
        highgui::imshow("mask", &mask)?;
        highgui::imshow("result", &result)?;

        // 按键：q 或 ESC 退出
        let key = highgui::wait_key(1)?;
        if key == 113 || key == 27 {
            break;
        }
    }

    Ok(())
}

 
#[test]   // TODO
fn test_camera_detect_color_loop() -> Result<()> {
    // ===== 读配置 =====
    let config = crate::utils::device_config_util::get_config()?;
    let config = config.color_camera_config;
    let camera_filename = config.color_camera;

    // ===== 构造 items（直接用 config 里的）=====
    let mut items: Vec<(&'static str, [i32; 6])> = Vec::new();
    for c in &config.colors {
        match c.as_str() {
            "red" => items.push(("red", config.hsv_red)),
            "blue" => items.push(("blue", config.hsv_blue)),
            "green" => items.push(("green", config.hsv_green)),
            "black" => items.push(("black", config.hsv_black)),
            "white" => items.push(("white", config.hsv_white)),
            _ => {}
        }
    }

    assert!(!items.is_empty(), "items 为空，检查 config.colors");

    // ===== 打开摄像头 =====
    let mut cam = videoio::VideoCapture::from_file(&camera_filename, videoio::CAP_ANY)?;
    assert!(
        videoio::VideoCapture::is_opened(&cam)?,
        "无法打开摄像头：{}",
        camera_filename
    );

    highgui::named_window("camera_test", highgui::WINDOW_NORMAL)?;

    let mut frame = Mat::default();

    loop {
        cam.read(&mut frame)?;
        if frame.empty() {
            break;
        }

        // ===== 调你的检测函数 =====
        let result = detect_top_circle_color(&frame, &mut items)?;

        // ===== 把颜色写到画面上 =====
        let mut show = frame.clone();
        let text = match result {
            Some(color) => format!("color: {}", color),
            None => "color: none".to_string(),
        };

        imgproc::put_text(
            &mut show,
            &text,
            core::Point::new(30, 50),
            imgproc::FONT_HERSHEY_SIMPLEX,
            1.2,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            false,
        )?;

        highgui::imshow("camera_test", &show)?;

        // q 退出
        let key = highgui::wait_key(1)?;
        if key == 113 {
            break;
        }
    }

    Ok(())
}