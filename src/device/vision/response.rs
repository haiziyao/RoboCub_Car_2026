use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use rppal::{
    gpio::{Gpio, OutputPin},
    uart::{Parity, Uart},
};

use crate::config::UartParam;

use super::config::{arg_map, u8_value};

#[derive(Debug, Clone)]
pub struct ResponseDeviceConfig {
    pub uart: UartDeviceConfig,
    pub lights: Option<LightConfig>,
}

#[derive(Debug, Clone)]
pub struct UartDeviceConfig {
    pub serial: String,
    pub baud: u32,
    pub data_bit: u8,
    pub stop_bit: u8,
    pub parity_bit: bool,
}

#[derive(Debug, Clone)]
pub struct LightConfig {
    pub color_light_pin: u8,
    pub qr_light_pin: u8,
    pub gpio_light_pin: u8,
}

#[derive(Copy, Clone, Debug)]
pub enum TaskLightKind {
    Color,
    Qr,
}

pub struct LightSession {
    pins: Vec<OutputPin>,
}

impl Drop for LightSession {
    fn drop(&mut self) {
        for pin in &mut self.pins {
            pin.set_high();
        }
    }
}

impl ResponseDeviceConfig {
    pub fn from_args_with_uart(args: &[String], uart: UartDeviceConfig) -> Result<Self> {
        let map = arg_map(args);
        Ok(Self {
            uart,
            lights: LightConfig::from_map(&map)?,
        })
    }
}

impl UartDeviceConfig {
    pub fn new(
        serial: String,
        baud: u32,
        data_bit: u8,
        stop_bit: u8,
        parity_bit: bool,
    ) -> Result<Self> {
        let config = Self {
            serial,
            baud,
            data_bit,
            stop_bit,
            parity_bit,
        };
        validate_uart_config(&config)?;
        Ok(config)
    }

    pub fn from_param(param: &UartParam) -> Result<Self> {
        Self::new(
            param.serial.clone(),
            param.baud,
            param.data_bit,
            param.stop_bit,
            param.parity_bit,
        )
    }

    pub fn parity(&self) -> Parity {
        if self.parity_bit {
            Parity::Even
        } else {
            Parity::None
        }
    }

    pub fn open_uart(&self) -> Result<Uart> {
        let mut uart = Uart::with_path(
            self.serial.clone(),
            self.baud,
            self.parity(),
            self.data_bit,
            self.stop_bit,
        )
        .with_context(|| format!("failed to open uart {}", self.serial))?;
        uart.set_read_mode(0, Duration::from_millis(100))?;
        Ok(uart)
    }
}

fn validate_uart_config(config: &UartDeviceConfig) -> Result<()> {
    if config.serial.is_empty() {
        return Err(anyhow!("invalid parameter `serial`: value is empty"));
    }
    if config.baud == 0 {
        return Err(anyhow!("invalid parameter `baud`: must be greater than 0"));
    }
    if !(5..=8).contains(&config.data_bit) {
        return Err(anyhow!("invalid parameter `data_bit`: must be in 5..=8"));
    }
    if !(1..=2).contains(&config.stop_bit) {
        return Err(anyhow!("invalid parameter `stop_bit`: must be 1 or 2"));
    }
    Ok(())
}

impl LightConfig {
    pub fn from_map(map: &std::collections::HashMap<String, String>) -> Result<Option<Self>> {
        let has_light = map.contains_key("color_light_pin")
            || map.contains_key("qr_light_pin")
            || map.contains_key("gpio_light_pin");
        if !has_light {
            return Ok(None);
        }

        Ok(Some(Self {
            color_light_pin: u8_value(map, "color_light_pin")?,
            qr_light_pin: u8_value(map, "qr_light_pin")?,
            gpio_light_pin: u8_value(map, "gpio_light_pin")?,
        }))
    }
}

pub fn begin_light_session(
    response: &ResponseDeviceConfig,
    task: TaskLightKind,
) -> Result<Option<LightSession>> {
    let Some(config) = response.lights.as_ref() else {
        return Ok(None);
    };

    let gpio = Gpio::new().context("failed to access gpio")?;
    let mut pins = Vec::new();

    let mut gpio_pin = gpio
        .get(config.gpio_light_pin)
        .with_context(|| format!("failed to access gpio pin {}", config.gpio_light_pin))?
        .into_output();
    gpio_pin.set_low();
    pins.push(gpio_pin);

    let task_pin_number = match task {
        TaskLightKind::Color => config.color_light_pin,
        TaskLightKind::Qr => config.qr_light_pin,
    };
    let mut task_pin = gpio
        .get(task_pin_number)
        .with_context(|| format!("failed to access gpio pin {}", task_pin_number))?
        .into_output();
    task_pin.set_low();
    pins.push(task_pin);

    Ok(Some(LightSession { pins }))
}

pub fn send_uart_line(config: &UartDeviceConfig, value: &str) -> Result<()> {
    let mut uart = config.open_uart()?;
    uart.set_write_mode(true)?;
    write_uart_all(&mut uart, value.as_bytes())?;
    write_uart_all(&mut uart, b"\n")?;
    uart.drain()?;
    Ok(())
}

pub fn send_response_line(response: &ResponseDeviceConfig, value: &str) -> Result<()> {
    send_uart_line(&response.uart, value)
}

fn write_uart_all(uart: &mut Uart, mut bytes: &[u8]) -> Result<()> {
    while !bytes.is_empty() {
        let written = uart.write(bytes)?;
        if written == 0 {
            return Err(anyhow!("uart write returned 0 bytes"));
        }
        bytes = &bytes[written..];
    }
    Ok(())
}
