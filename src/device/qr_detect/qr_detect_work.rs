use anyhow::{Result};
use opencv::core::MatTraitConstManual;
use opencv::highgui;
use crate::{config::device_config::QrCameraConfig, device::camera::register_qr_camera};
use crate::utils::cv_util::bgr_to_gray;
use opencv::{
    core::{self, MatTraitConst}, videoio::VideoCaptureTrait,
};
 


pub fn work(config:QrCameraConfig)-> Result<i32>{
    let mut cam = register_qr_camera(config.clone())?;
    loop {
        let mut frame = core::Mat::default();
        cam.read(&mut frame)?;
        if frame.empty() {continue;}

        let mut processed_frame = frame_pre_process(&mut frame)?;
        let content: String;
        let num:i32;


        if config.debug_model {
            content = decode_qr_debugging(&mut processed_frame)?; 
        } else {
            content = decode_qr(&mut processed_frame)?;
        }
      

        if !content.is_empty()  {
            num = content.parse()?;
            return anyhow::Ok(num)
        }
        
        if config.debug_model{
            highgui::imshow("nothing", &processed_frame)?;
            let key = highgui::wait_key(1)?;  
            if key == 27 { break; }
        }           
    }
    Ok(-1)
}

fn frame_pre_process(frame:&mut core::Mat) ->Result<core::Mat>{
    // let thresh = 100.0;
    // let maxval = 255.0;
    // let mut binary = threshold(frame, thresh, maxval)?;

    let gray = bgr_to_gray(frame)?;
    Ok(gray)
}

fn decode_qr(processed_frame: &core::Mat) -> Result<String> {
    let size = processed_frame.size()?;
    let width = size.width as usize;
    let height = size.height as usize;

    let data = processed_frame.data_bytes()?;

    let mut decoder = quircs::Quirc::default();
    let codes = decoder.identify(width, height, &data[..width * height]);
    let mut content = String::new();

    for (_i, code_res) in codes.enumerate() {

        let code = match code_res {
            Ok(c) => c,
            Err(_) =>  continue
        };

        let decoded = match code.decode() {
            Ok(d) => d,
            Err(_) => continue
        };

        match std::str::from_utf8(&decoded.payload) {
            Ok(text) => {
                content = text.to_string();
                return Ok(content);
            },
            Err(_) => println!("[QR] payload bytes: {:?}", decoded.payload),
        }
    }

    anyhow::Ok(content)
}

fn decode_qr_debugging(processed_frame: &core::Mat) -> Result<String> {
    let size = processed_frame.size()?;
    let width = size.width as usize;
    let height = size.height as usize;

    let data = processed_frame.data_bytes()?;

    let mut decoder = quircs::Quirc::default();
    let codes = decoder.identify(width, height, &data[..width * height]);
    let  content ;

    for (i, code_res) in codes.enumerate() {

        let code = match code_res {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[QR] extract failed #{i}: {:?}", e);
                continue;
            }
        };

        let decoded = match code.decode() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[QR] decode failed #{i}: {:?}", e);
                continue;
            }
        };

        match std::str::from_utf8(&decoded.payload) {
            Ok(text) => {
                println!("[QR] {}", text);
                content = text.to_string();
                return Ok(content);
            },
            Err(_) => println!("[QR] payload bytes: {:?}", decoded.payload),
        }
    }

    anyhow::Ok(String::new())
}

 