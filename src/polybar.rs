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
    icon: &'static str,
}

impl PolybarOutput {
    pub fn new(command_channel: Sender<RotelCommand>, response_channel: Receiver<RotelResponse>,) -> Self {
        Self {command_channel, response_channel, icon: "Rotel"}
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
        let mut status = self.query_status().await?;
        self.print_status(&status);
        while let Some(response) = self.response_channel.recv().await {
            match response {
                RotelResponse::Volume(val) => status.volume = val,
                RotelResponse::Mute(val) => status.mute = val,
                _ => continue,
            }
            self.print_status(&status);
        }
        Ok(())
    }

    fn print_status(&self, status: &RotelStatus) {
        let color = if status.mute == StateToggle::On {
            "f00"
        } else {
            "fff"
        };
        println!("{icon}: %{{F#{color}}}{value}%{{F-}}", icon = self.icon, color = color, value = status.volume);
    }
}
