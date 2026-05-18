//! 异步视频流引擎层：负责采集帧、运行检测，并通过双通道消息回传。
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use opencv::{
    core,
    imgcodecs,
    imgproc,
    prelude::*,
    videoio,
    types,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::path::Path;

use crate::config::device_config::ColorCameraConfig;
use crate::device::camera;
use crate::utils::cv_util::hsv_scalar_factory;

/// 传统 HSV 阈值检测所需的参数集合（算法配置载体）。
#[derive(Clone, Debug)]
pub struct ColorDetectParams {
    pub hsv_lower: core::Scalar,
    pub hsv_upper: core::Scalar,
    pub min_area: f64,
    pub encoding: ImageEncoding,
}

/// 编码格式：用于将标注后的图像编码为可传输的字节流。
#[derive(Copy, Clone, Debug)]
pub enum ImageEncoding {
    Jpeg,
    Png,
}

impl ImageEncoding {
    fn extension(self) -> &'static str {
        match self {
            ImageEncoding::Jpeg => ".jpg",
            ImageEncoding::Png => ".png",
        }
    }
}

impl ColorDetectParams {
    /// 从 HSV 六元组快速构造检测参数（便于快速标定与模拟）。
    pub fn from_hsv_range(hsv: [i32; 6], min_area: f64, encoding: ImageEncoding) -> Result<Self> {
        let (hsv_lower, hsv_upper) = hsv_scalar_factory(hsv)?;
        Ok(Self {
            hsv_lower,
            hsv_upper,
            min_area,
            encoding,
        })
    }
}

/// 根据配置中的颜色名生成对应 HSV 区间参数。
pub fn params_from_config_color(
    config: &ColorCameraConfig,
    color_name: &str,
    min_area: f64,
    encoding: ImageEncoding,
) -> Result<ColorDetectParams> {
    let hsv = match color_name {
        "red" => config.hsv_red,
        "blue" => config.hsv_blue,
        "green" => config.hsv_green,
        "black" => config.hsv_black,
        "white" => config.hsv_white,
        _ => anyhow::bail!("unknown color name: {color_name}"),
    };
    ColorDetectParams::from_hsv_range(hsv, min_area, encoding)
}

/// 将帧和参数打包，便于异步线程解耦处理。
pub struct ColorDetectTask {
    pub frame: core::Mat,
    pub params: ColorDetectParams,
}

/// 循环帧源：用于周期性推送检测任务的线程入口。
pub struct LoopSource;

impl LoopSource {
    /// 从相机持续读取帧并投递任务，保持稳定节拍。
    pub fn start(
        tx: mpsc::Sender<ColorDetectTask>,
        config: ColorCameraConfig,
        params: ColorDetectParams,
    ) -> thread::JoinHandle<Result<()>> {
        thread::spawn(move || {
            let mut cam = camera::register_color_camera(config)
                .context("LoopSource failed to open color camera")?;
            if !cam.is_opened()? {
                anyhow::bail!("LoopSource camera not opened");
            }

            let interval = Duration::from_secs(5);
            loop {
                let started = Instant::now();

                let mut frame = core::Mat::default();
                cam.read(&mut frame)?;
                if frame.empty() {
                    thread::sleep(interval);
                    continue;
                }

                if tx
                    .send(ColorDetectTask {
                        frame,
                        params: params.clone(),
                    })
                    .is_err()
                {
                    break;
                }

                // 保持 5 秒节拍，避免采集过快导致下游拥塞。
                let elapsed = started.elapsed();
                if elapsed < interval {
                    thread::sleep(interval - elapsed);
                }
            }

            Ok(())
        })
    }
}

/// 定时触发源：用于低频或一次性采样场景。
pub struct TimerSource;

impl TimerSource {
    /// 延迟启动后抓取一帧并发送任务。
    pub fn start(
        tx: mpsc::Sender<ColorDetectTask>,
        config: ColorCameraConfig,
        params: ColorDetectParams,
    ) -> thread::JoinHandle<Result<()>> {
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(30));

            let mut cam = camera::register_color_camera(config)
                .context("TimerSource failed to open color camera")?;
            if !cam.is_opened()? {
                anyhow::bail!("TimerSource camera not opened");
            }

            let mut frame = core::Mat::default();
            cam.read(&mut frame)?;
            if frame.empty() {
                return Ok(());
            }

            let _ = tx.send(ColorDetectTask { frame, params });
            Ok(())
        })
    }
}

/// 颜色检测状态，用于区分空帧/无轮廓等情况。
#[derive(Debug)]
pub enum DetectStatus {
    Ok,
    EmptyFrame,
    NoContours,
}

/// 颜色检测结果：包含 Base64 图像与检测统计信息。
#[derive(Debug)]
pub struct DetectColorResult {
    pub status: DetectStatus,
    pub image_base64: String,
    pub box_count: usize,
}

/// 扩展封装：允许在结果外附加任务 ID 等元数据。
pub struct ColorDetectEnvelope {
    pub task_id: String,
    pub result: DetectColorResult,
}

/// 基于 HSV 阈值的全帧检测（用于传统方案或快速验证）。
pub fn detect_color(mut frame_bgr: core::Mat, params: &ColorDetectParams) -> Result<DetectColorResult> {
    if frame_bgr.empty() {
        return Ok(DetectColorResult {
            status: DetectStatus::EmptyFrame,
            image_base64: String::new(),
            box_count: 0,
        });
    }

    // 转到 HSV 空间并生成目标颜色的二值掩码。
    let mut hsv = core::Mat::default();
    imgproc::cvt_color(&frame_bgr, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

    let mut mask = core::Mat::default();
    core::in_range(&hsv, &params.hsv_lower, &params.hsv_upper, &mut mask)?;

    let mut contours = types::VectorOfVectorOfPoint::new();
    let mut hierarchy = core::Mat::default();
    imgproc::find_contours(
        &mask,
        &mut contours,
        imgproc::RETR_EXTERNAL,
        imgproc::CHAIN_APPROX_SIMPLE,
        core::Point::new(0, 0),
    )?;

    let mut box_count = 0usize;
    for contour in contours {
        // 过滤小面积噪声后再画框，降低误检。
        let area = imgproc::contour_area(&contour, false)?;
        if area < params.min_area {
            continue;
        }

        let rect = imgproc::bounding_rect(&contour)?;
        imgproc::rectangle(
            &mut frame_bgr,
            rect,
            core::Scalar::new(0.0, 255.0, 0.0, 0.0),
            2,
            imgproc::LINE_8,
            0,
        )?;
        box_count += 1;
    }

    let status = if box_count > 0 {
        DetectStatus::Ok
    } else {
        DetectStatus::NoContours
    };

    // 编码标注后的图像，便于 Web UI 直接渲染。
    let image_base64 = encode_base64_image(&frame_bgr, params.encoding)?;

    Ok(DetectColorResult {
        status,
        image_base64,
        box_count,
    })
}

// 将图像编码为 Base64，供 Web UI 直接渲染。
fn encode_base64_image(frame_bgr: &core::Mat, encoding: ImageEncoding) -> Result<String> {
    let mut buf = types::VectorOfu8::new();
    let mut params = types::VectorOfi32::new();

    match encoding {
        ImageEncoding::Jpeg => {
            params.push(imgcodecs::IMWRITE_JPEG_QUALITY);
            params.push(85);
        }
        ImageEncoding::Png => {
            params.push(imgcodecs::IMWRITE_PNG_COMPRESSION);
            params.push(3);
        }
    }

    imgcodecs::imencode(encoding.extension(), frame_bgr, &mut buf, &params)?;
    Ok(general_purpose::STANDARD.encode(buf.as_slice()))
}

/// 定时发送单帧 Base64 图像（示例/测试用）。
pub fn start_timer_source(tx: mpsc::Sender<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(30));

        let params = match dummy_params() {
            Ok(params) => params,
            Err(err) => {
                eprintln!("TimerSource params error: {err}");
                return;
            }
        };

        let frame = match read_test_image("test_color.jpg") {
            Ok(frame) => frame,
            Err(err) => {
                eprintln!("TimerSource image error: {err}");
                return;
            }
        };

        let result = match detect_color(frame, &params) {
            Ok(result) => result,
            Err(err) => {
                eprintln!("TimerSource detect_color error: {err}");
                return;
            }
        };

        let base64_string = result.image_base64;
        if let Err(err) = tx.send(base64_string) {
            eprintln!("TimerSource send failed: {err}");
        }
    })
}

/// 异步主循环：读取视频帧，标注 ROI/颜色结果，并通过双通道发送。
pub fn start_loop_source(
    tx: std::sync::mpsc::Sender<(String, String)>,
    source_path: String,
    config: crate::config::device_config::ColorCameraConfig // 接收组长的配置
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // 根据输入路径决定读取摄像头或文件，避免阻塞主线程。
        let mut cam = if let Ok(idx) = source_path.parse::<i32>() {
            opencv::videoio::VideoCapture::new(idx, opencv::videoio::CAP_V4L2).unwrap()
        } else {
            opencv::videoio::VideoCapture::from_file(&source_path, opencv::videoio::CAP_FFMPEG).unwrap()
        };

        loop {
            let mut frame = opencv::core::Mat::default();
            if let Err(_) = opencv::videoio::VideoCapture::read(&mut cam, &mut frame) { continue; }
            if frame.empty() {
                // 文件源读到结尾时回到起始帧，实现循环播放。
                let _ = cam.set(opencv::videoio::CAP_PROP_POS_FRAMES, 0.0);
                continue;
            }

            // 1) 生成圆形 ROI 掩码，只在圆内做颜色统计。
            let (_roi, circle_mask) = crate::utils::cv_util::roi_circle_mask(&frame, config.radius_ratio).unwrap();
            
            // 2) 计算 ROI 内主色与占比（核心算法层）。
            let (color_name, ratio) = crate::device::color_detect::color_detect_work::detect_color_in_circle_mask(&frame, &circle_mask, &config).unwrap();

            // 3) 在帧上叠加 ROI 和识别结果，便于 Web UI 可视化。
            let _ = crate::device::color_detect::color_detect_work::draw_debug_info(&mut frame, &color_name, ratio, config.radius_ratio);

            // 4) 编码 Base64，作为 Web UI 的图像数据通道。
            let mut buf = opencv::core::Vector::<u8>::new();
            opencv::imgcodecs::imencode(".jpg", &frame, &mut buf, &opencv::core::Vector::new()).unwrap();
            let image_base64 = base64::encode(buf.as_slice()); 

            // 5) 双通道解耦：Base64 给 Web UI，颜色字符串给 UART 控制逻辑。
            if let Err(_) = tx.send((image_base64, color_name)) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}

fn dummy_params() -> Result<ColorDetectParams> {
    ColorDetectParams::from_hsv_range([0, 10, 120, 255, 80, 255], 80.0, ImageEncoding::Jpeg)
}

fn read_test_image(path: &str) -> Result<core::Mat> {
    if !Path::new(path).exists() {
        anyhow::bail!("test image not found: {path}");
    }

    let frame = imgcodecs::imread(path, imgcodecs::IMREAD_COLOR)?;
    if frame.empty() {
        anyhow::bail!("loaded image is empty: {path}");
    }

    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{bail, Result};
    use opencv::imgcodecs::{imread, IMREAD_COLOR};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_detect_color_from_file() -> Result<()> {
        let image_path = "test_color.jpg";
        if !Path::new(image_path).exists() {
            bail!("test image not found: {image_path} (place it in the project root)");
        }

        let frame = imread(image_path, IMREAD_COLOR)?;
        if frame.empty() {
            bail!("loaded image is empty: {image_path}");
        }

        // 红色的 HSV 示例区间，实际使用请根据测试图像调整。
        let params = ColorDetectParams::from_hsv_range(
            [0, 10, 120, 255, 80, 255],
            80.0,
            ImageEncoding::Jpeg,
        )?;

        let result = detect_color(frame, &params)?;
        fs::write("output_base64.txt", result.image_base64)?;
        Ok(())
    }
}
