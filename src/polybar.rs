use crate::controller::protocol::{RotelResponse, StateToggle, Volume, RotelCommand};
use tokio::sync::mpsc::{Receiver, Sender};

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

    // async fn query_status(&self) -> RotelStatus {
    //
    // }

    pub async fn run(&mut self) {
        let icon = "R";
        while let Some(response) = self.response_channel.recv().await {
            println!("R {:?}", response)
        }
    }
}
