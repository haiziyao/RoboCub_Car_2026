use std::time::Duration;
use std::io::{self, Write, Read};
use std::thread;
use anyhow::Result;
#[test]

fn try_fn() -> Result<()> {
    // 打开串口
    let mut port = serialport::new("/dev/ttyV0", 9600)
        .timeout(Duration::from_millis(0)) // 设置超时
        .open()
        .expect("Failed to open port");

    // 主线程负责发送数据
    //let output = "This is a test. This is only a test.".as_bytes();
    //port.write(output).expect("Write failed!");

    // 启动一个子线程来读取串口数据
    thread::spawn(move || {
        let mut serial_buf: Vec<u8> = vec![0; 32]; // 创建缓冲区
        loop {
            match port.read(&mut serial_buf) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        println!("Read {} bytes: {:?}", bytes_read, &serial_buf[..bytes_read]);
                    }
                }
                Err(e) => {
                    println!("Failed to read data: {:?}", e);
                    break;
                }
            }
        }
    });

    // 主线程阻塞
    loop {
        // 可以在主线程中执行其他任务
        // 如果你希望主线程一直阻塞，可以放置一个空的循环，或者加上逻辑
        // 可以考虑加上其他终止条件
    }

    Ok(())
}