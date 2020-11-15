//! Rotel line protocol
//!
//! This module implements the rotel IP/RS232 ASCII protocol version 2.
//! It is based and tested on the RA-1572.
//! Behaviour should be similar on other models of the same family.

use self::constants::*;
use color_eyre::eyre::{eyre, Report, Result};
use std::convert::TryFrom;
use std::fmt::Formatter;
use std::num::ParseIntError;
use std::str::FromStr;

pub mod constants;

pub trait ToRotel {
    fn to_rotel(&self) -> &[u8];
}

#[derive(Debug)]
pub enum StateToggle {
    On,
    Off,
}

impl FromStr for StateToggle {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(eyre!("wrong value for state: {}", s)),
        }
    }
}

impl std::fmt::Display for StateToggle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::On => "on",
            Self::Off => "off",
        };
        write!(f, "{}", output)
    }
}

#[derive(Debug)]
pub struct Volume(u8);

impl TryFrom<u8> for Volume {
    type Error = Report;

    fn try_from(value: u8) -> Result<Self> {
        if value > MAX_VOLUME {
            Err(eyre!("value for volume is out of bounds: {}", value))
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Volume> for u8 {
    fn from(volume: Volume) -> u8 {
        volume.0
    }
}

impl FromStr for Volume {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse().map_err(|err: ParseIntError| {
            eyre!(format!("Failed to parse volume: {}", err.to_string()))
        })?;
        Self::try_from(value)
    }
}

impl std::fmt::Display for Volume {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Dimmer(u8);

impl TryFrom<u8> for Dimmer {
    type Error = Report;

    fn try_from(value: u8) -> Result<Self> {
        if value > MAX_DIMMER {
            Err(eyre!("value for volume is out of bounds: {}", value))
        } else {
            Ok(Self(value))
        }
    }
}

impl From<Dimmer> for u8 {
    fn from(volume: Dimmer) -> u8 {
        volume.0
    }
}

impl FromStr for Dimmer {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse().map_err(|err: ParseIntError| {
            eyre!(format!("Failed to parse dimmer: {}", err.to_string()))
        })?;
        Self::try_from(value)
    }
}

impl std::fmt::Display for Dimmer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Feedback request commands
///
/// These commands are used to query the state of the amp. They do not change the state.
pub enum RotelQuery {
    Power,
    Volume,
    Source,
    Mute,
    Frequency,
    Speaker,
    Dimmer,
    Version,
    Model,
}

impl From<RotelQuery> for &'static str {
    fn from(query: RotelQuery) -> Self {
        use RotelQuery::*;
        match query {
            Power => "power?",
            Volume => "volume?",
            Source => "source?",
            Mute => "mute?",
            Frequency => "freq?",
            Speaker => "speaker?",
            Dimmer => "dimmer?",
            Version => "version?",
            Model => "model?",
        }
    }
}

impl From<RotelQuery> for &[u8] {
    fn from(query: RotelQuery) -> Self {
        let msg: &str = query.into();
        msg.as_bytes()
    }
}

/// Control commands
///
/// These commands are used to change the state of the amp.
/// For commands taking a boolean, `true` means 'on' and `false` means 'off'.
pub enum RotelCommand {
    Mute(StateToggle),
    Power(StateToggle),
    Volume(Volume),
}

impl RotelCommand {
    pub fn build_command(&self) -> String {
        format!(
            "{}!",
            match self {
                RotelCommand::Mute(value) => format!("mute_{}", value),
                RotelCommand::Power(value) => format!("power_{}", value),
                RotelCommand::Volume(volume) => format!("vol_{:02}", volume.0),
            }
        )
    }
}

/// Messages received from the amp
#[derive(Debug)]
pub enum RotelResponse {
    Power(StateToggle),
    Volume(Volume),
    Mute(StateToggle),
    Unknown(String),
}

impl TryFrom<&[u8]> for RotelResponse {
    type Error = Report;

    fn try_from(in_msg: &[u8]) -> Result<Self> {
        let msg =
            std::str::from_utf8(in_msg).map_err(|err| eyre!("message is not UTF-8: {:?}", err))?;
        let delim_index = msg
            .find('=')
            .ok_or_else(|| eyre!("received message doesn't match expected pattern"))?;
        let (cmd, value) = msg.split_at(delim_index);
        let value = &value[1..];
        let rotel_message = match cmd {
            "volume" => RotelResponse::Volume(value.parse::<Volume>()?),
            "power" => RotelResponse::Power(value.parse::<StateToggle>()?),
            "mute" => RotelResponse::Mute(value.parse::<StateToggle>()?),
            _ => RotelResponse::Unknown(msg.into()),
        };
        Ok(rotel_message)
    }
}
