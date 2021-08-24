use crate::controller::protocol::{RotelCommand, RotelQuery, RotelResponse, StateToggle, Volume};
use color_eyre::eyre::{eyre, Result};
use color_eyre::Report;
use std::str::FromStr;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
struct RotelStatus {
    mute: StateToggle,
    volume: Volume,
}

pub struct OutputFormatter {
    pub before: String,
    pub after: String,
}

impl OutputFormatter {
    fn format<D: std::fmt::Display>(self, value: D) -> String {
        format!(
            "{before}{value}{after}",
            before = self.before,
            after = self.after,
            value = value
        )
    }
}

impl FromStr for OutputFormatter {
    type Err = Report;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let (before, after) = input
            .split_once("{value}")
            .ok_or_else(|| eyre!("Format must contain `{value}`"))?;
        Ok(Self {
            before: before.to_owned(),
            after: after.to_owned(),
        })
    }
}

pub struct PolybarOutput {
    command_channel: Sender<RotelCommand>,
    response_channel: Receiver<RotelResponse>,
    volume_formatter: OutputFormatter,
    mute_formatter: OutputFormatter,
}

impl PolybarOutput {
    pub fn new(
        command_channel: Sender<RotelCommand>,
        response_channel: Receiver<RotelResponse>,
        volume_formatter: OutputFormatter,
        mute_formatter: OutputFormatter,
    ) -> Self {
        Self {
            command_channel,
            response_channel,
            volume_formatter: volume_formatter,
            mute_formatter: mute_formatter,
        }
    }

    async fn query_status(&mut self) -> Result<RotelStatus> {
        self.command_channel
            .send(RotelCommand::Get(RotelQuery::Volume))
            .await?;
        self.command_channel
            .send(RotelCommand::Get(RotelQuery::Mute))
            .await?;
        let mut mute: Option<StateToggle> = None;
        let mut volume: Option<Volume> = None;
        while let Some(response) = self.response_channel.recv().await {
            match response {
                RotelResponse::Volume(val) => volume = Some(val),
                RotelResponse::Mute(val) => mute = Some(val),
                _ => continue,
            }
            if mute.is_some() && volume.is_some() {
                return Ok(RotelStatus {
                    mute: mute.unwrap(),
                    volume: volume.unwrap(),
                });
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
        let formatter = if status.mute == StateToggle::On {
            &self.mute_formatter
        } else {
            &self.volume_formatter
        };
        println!(
            "{before}{value}{after}",
            before = formatter.before,
            after = formatter.after,
            value = status.volume
        )
    }
}
