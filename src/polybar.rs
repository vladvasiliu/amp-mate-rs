use crate::controller::protocol::{RotelResponse, StateToggle, Volume, RotelCommand, RotelQuery};
use color_eyre::eyre::{eyre, Result};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
struct RotelStatus {
    mute: StateToggle,
    volume: Volume,
}

pub struct PolybarOutput {
    command_channel: Sender<RotelCommand>,
    response_channel: Receiver<RotelResponse>,
}

impl PolybarOutput {
    pub fn new(command_channel: Sender<RotelCommand>, response_channel: Receiver<RotelResponse>,) -> Self {
        Self {command_channel, response_channel}
    }

    async fn query_status(&mut self) -> Result<RotelStatus> {
        self.command_channel.send(RotelCommand::Get(RotelQuery::Volume)).await?;
        self.command_channel.send(RotelCommand::Get(RotelQuery::Mute)).await?;
        let mut mute: Option<StateToggle> = None;
        let mut volume: Option<Volume> = None;
        while let Some(response) = self.response_channel.recv().await {
            match response {
                RotelResponse::Volume(val) => volume = Some(val),
                RotelResponse::Mute(val) => mute = Some(val),
                _ => continue,
            }
            if mute.is_some() && volume.is_some() {
                return Ok(RotelStatus { mute: mute.unwrap(), volume: volume.unwrap() });
            }
        }
        Err(eyre!("Failed to get initial status. Channel closed."))
    }

    pub async fn run(&mut self) -> Result<()> {
        let icon = "R";
        let mut status = self.query_status().await?;
        println!("R: {:?}", status);
        while let Some(response) = self.response_channel.recv().await {
            match response {
                RotelResponse::Volume(val) => status.volume = val,
                RotelResponse::Mute(val) => status.mute = val,
                _ => continue,
            }
            println!("{}: {:?}", icon, status);
        }
        Ok(())
    }
}
