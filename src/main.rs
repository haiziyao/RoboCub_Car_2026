mod config;
mod device;
mod utils;
mod web;

use crate::config::config_cli::register_config_cli;
use crate::{device::qr_detect::qr_detect_work, utils::device_config_util::get_config};
use crate::device::color_detect::color_detect_work;
use anyhow::Result;
use std::sync::{Arc, RwLock};
use std::thread;
//use crate::device::gpio::gpio_work::{receive_line_loop,register_gpio,send_line,register_lights};
 
fn main() -> Result<()> {
    // 命令行给参
    let _ = register_config_cli()?;
    // 读取配置
    let my_config = get_config()?; 
    let (gpio_config,qr_config,color_config,light_config) = 
    (my_config.gpio_config,my_config.qr_camera_config,my_config.color_camera_config, my_config.light_config);
 //   let gpio_config_main = gpio_config.clone();
    

    // 双通道解耦：Base64 图像给 Web UI，颜色字符串给 UART 控制逻辑。
    let (tx, rx): (
        std::sync::mpsc::Sender<(String, String)>,
        std::sync::mpsc::Receiver<(String, String)>,
    ) = std::sync::mpsc::channel();
    let latest_image = Arc::new(RwLock::new(String::new()));

    let web_config = match web::WebConfig::from_file("../config/web.yaml") {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Web config error: {err}. Using defaults.");
            web::WebConfig::default()
        }
    };
    let _web_handle = if web_config.on {
        Some(web::start_server(web_config, Arc::clone(&latest_image)))
    } else {
        None
    };
// --- 智能读取终端参数 ---
    let args: Vec<String> = std::env::args().collect();
    // 只需要拿视频路径就行了，不需要自己输入颜色参数了
    let source_path = if args.len() > 1 { args[1].clone() } else { "0".to_string() };

    // 注意这里：不再传字符串，而是把你们项目中本来就有的 color_config 传进去！
    // 异步视频流引擎启动：主线程只负责路由，不参与重计算。
    crate::device::color_detect::color_detect_task::start_loop_source(tx.clone(), source_path, color_config.clone());
    //crate::device::color_detect::color_detect_task::start_timer_source(tx.clone());
    /*
    let handle = thread::spawn(move || {
        let mut uart = register_gpio(gpio_config).expect("Failed to initialize UART");
        receive_line_loop(&mut uart, tx).expect("Failed to receive data");
    });
    */
    // 处理消息
    //let mut uart = register_gpio(gpio_config_main).expect("Failed to initialize UART");
    //let (mut color_pin,mut qr_pin,mut gpio_pin) = register_lights(light_config)?;
    // 主线程消息路由：Web UI 与 UART 逻辑彼此独立，互不阻塞。
    loop {
        match rx.recv() {
            Ok((received_base64, detected_color)) => {
                // 硬件通道：输出 UART 指令（后续可替换为真实串口发送）。
                println!("Hardware UART instruction: {}", detected_color);
                // Web 通道：更新共享内存，供 Web UI 定时刷新读取。
                if let Ok(mut latest) = latest_image.write() {
                    *latest = received_base64;
                }
            }
            Err(e) => {
                eprintln!("Failed to receive data: {}", e);
                break;
            }
        }
    }

    // 等待子线程完成
    //handle.join().expect("Thread panicked");

    Ok(())
}




#[test]
fn test_color_detect() -> Result<()> {
    let my_config = get_config()?; 
    let color_name = color_detect_work::work(my_config.color_camera_config)?;
    println!("找到最佳颜色 {}",color_name );
   Ok(())
}

#[test]
fn test_qr_detect() -> Result<()>{
    let my_config = get_config()?; 
    let task_num = qr_detect_work::work(my_config.qr_camera_config)?;
    println!("已经识别二维码 {}",task_num);
   Ok(()) 
}
