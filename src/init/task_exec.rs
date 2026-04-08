use anyhow::{anyhow, Context, Result};
use tokio::sync::mpsc::Sender;
use tracing::info;

use crate::device::Device;
use crate::func::FunctionWorker;
use crate::web::WebMessage;

#[derive(Debug)]
pub struct TaskExecutor {
    sender: Sender<WebMessage>,
}

impl TaskExecutor {
    pub fn new(sender: Sender<WebMessage>) -> TaskExecutor {
        TaskExecutor { sender }
    }

    pub fn get_sender(&self) -> Sender<WebMessage> {
        self.sender.clone()
    }
}

pub fn execute_sync(device: Option<Device>, func: Option<FunctionWorker>) -> Result<WebMessage> {
    let mut device = device.unwrap_or(Device::None);
    let func_worker = func.ok_or_else(|| anyhow!("function not found"))?;

    let FunctionWorker {
        func_id,
        mut args,
        mut func,
    } = func_worker;

    info!(
        "{func_id}({args}) is running",
        func_id = func_id,
        args = args.join(" ")
    );

    let result = func(&mut args, &mut device);

    info!("{} has finished execution", func_id);
    Ok(result)
}

pub async fn execute(
    sender: Sender<WebMessage>,
    device: Option<Device>,
    func: Option<FunctionWorker>,
) -> Result<()> {
    let result = tokio::task::spawn_blocking(move || execute_sync(device, func))
        .await
        .context("blocking task join failed")??;

    sender
        .send(result)
        .await
        .context("failed to send web message")?;

    info!("task result sent to web channel");
    Ok(())
}
