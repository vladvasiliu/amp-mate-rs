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
    fn to_rotel(&self) -> String;
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("up") {
            Ok(Self::Up)
        } else if s.eq_ignore_ascii_case("down") {
            Ok(Self::Down)
        } else {
            Err(eyre!("wrong value for direction: {}", s))
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Up => write!(f, "up"),
            Self::Down => write!(f, "down"),
        }
    }
}

#[derive(Debug, PartialEq)]
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
pub enum Volume {
    Value(u8),
    Direction(Direction),
}

impl TryFrom<u8> for Volume {
    type Error = Report;

    fn try_from(value: u8) -> Result<Self> {
        if value > MAX_VOLUME {
            Err(eyre!("value for volume is out of bounds: {}", value))
        } else {
            Ok(Self::Value(value))
        }
    }
}

impl ToRotel for Volume {
    fn to_rotel(&self) -> String {
        match self {
            Self::Value(value) => format!("vol_{:02}", value),
            Self::Direction(Direction::Down) => format!("vol_dwn"),
            Self::Direction(Direction::Up) => format!("vol_up"),
        }
    }
}

//
// impl From<Volume> for u8 {
//     fn from(volume: Volume) -> u8 {
//         volume.0
//     }
// }

impl FromStr for Volume {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Self::try_from(value)
        } else if let Ok(value) = s.parse::<Direction>() {
            Ok(Self::Direction(value))
        } else {
            Err(eyre!("Failed to parse volume value: {}", s))
        }
    }
}

impl std::fmt::Display for Volume {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{}", value),
            Self::Direction(direction) => write!(f, "{}", direction),
        }
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
#[derive(Debug)]
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

impl ToRotel for RotelQuery {
    fn to_rotel(&self) -> String {
        use RotelQuery::*;
        let msg = match self {
            Power => "power?",
            Volume => "volume?",
            Source => "source?",
            Mute => "mute?",
            Frequency => "freq?",
            Speaker => "speaker?",
            Dimmer => "dimmer?",
            Version => "version?",
            Model => "model?",
        };
        msg.to_string()
    }
}

/// Control commands
///
/// These commands are used to change the state of the amp.
/// For commands taking a boolean, `true` means 'on' and `false` means 'off'.
#[derive(Debug)]
pub enum Change {
    Mute(StateToggle),
    Power(StateToggle),
    Volume(Volume),
}

impl Change {
    pub fn to_rotel(&self) -> String {
        format!(
            "{}!",
            match self {
                Change::Mute(value) => format!("mute_{}", value),
                Change::Power(value) => format!("power_{}", value),
                Change::Volume(volume) => volume.to_rotel(),
            }
        )
    }
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::Mute(mute) => format!("Mute: {}", mute),
            Self::Power(power) => format!("Power: {}", power),
            Self::Volume(volume) => format!("Volume: {}", volume),
        };
        write!(f, "Change: {}", msg)
    }
}

#[derive(Debug)]
pub enum RotelCommand {
    Set(Change),
    Get(RotelQuery),
}

impl RotelCommand {
    pub fn to_rotel(&self) -> String {
        match self {
            Self::Set(change) => change.to_rotel(),
            Self::Get(query) => query.to_rotel(),
        }
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
        let (cmd, value) = msg
            .split_once('=')
            .ok_or_else(|| eyre!("received message doesn't match expected pattern"))?;
        let rotel_message = match cmd {
            "volume" => RotelResponse::Volume(value.parse::<Volume>()?),
            "power" => RotelResponse::Power(value.parse::<StateToggle>()?),
            "mute" => RotelResponse::Mute(value.parse::<StateToggle>()?),
            _ => RotelResponse::Unknown(msg.into()),
        };
        Ok(rotel_message)
    }
}
