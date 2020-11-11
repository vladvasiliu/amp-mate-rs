//! Rotel line protocol
//!
//! This module implements the rotel IP/RS232 ASCII protocol version 2.
//! It is based and tested on the RA-1572.
//! Behaviour should be similar on other models of the same family.

use crate::controller::rotel::protocol::constants::*;
use crate::controller::rotel::protocol::patterns::*;
use color_eyre::eyre::{eyre, Report, Result};
use once_cell::sync::Lazy;
use regex::bytes::{Regex, RegexBuilder, RegexSet, RegexSetBuilder};
use std::convert::TryFrom;
use std::num::ParseIntError;
use std::str::FromStr;

pub mod constants;
pub mod patterns;

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
    Mute(bool),
    Power(bool),
    Volume(Volume),
}

impl RotelCommand {
    pub fn build_command(&self) -> String {
        format!(
            "{}!",
            match self {
                RotelCommand::Mute(value) => {
                    let state = if *value { "on" } else { "off" };
                    format!("mute_{}", state)
                }
                RotelCommand::Power(value) => {
                    let state = if *value { "on" } else { "off" };
                    format!("power_{}", state)
                }
                RotelCommand::Volume(volume) => format!("vol_{:02}", volume.0),
            }
        )
    }
}

impl From<Volume> for RotelCommand {
    fn from(volume: Volume) -> Self {
        Self::Volume(volume)
    }
}

/// Messages received from the amp
#[derive(Debug)]
pub enum RotelMessage {
    Power(bool),
    Volume(Volume),
    Mute(bool),
    Unknown(String),
}

impl TryFrom<&[u8]> for RotelMessage {
    type Error = Report;

    fn try_from(in_msg: &[u8]) -> Result<Self> {
        if !in_msg.is_ascii() {
            return Err(eyre!("Received unexpected non-ascii message".to_string(),));
        }
        let matches: Vec<_> = ROTEL_MSG_REGEX.matches(in_msg).into_iter().collect();
        if matches.len() != 1 {
            return Err(eyre!("Failed to parse message."));
        }
        ROTEL_MSG_READERS[0].parse(in_msg)
    }
}

pub struct RotelMessageParser {
    pub regex: Regex,
    pub parser: fn(&str) -> Result<RotelMessage>,
}

impl RotelMessageParser {
    pub fn new(pattern: &str, parser: fn(&str) -> Result<RotelMessage>) -> Result<Self> {
        let mut regex_builder = RegexBuilder::new(&format!("^{}$", pattern));
        regex_builder.case_insensitive(true);
        regex_builder.unicode(false);
        Ok(Self {
            regex: regex_builder.build()?,
            parser,
        })
    }

    pub fn parse(&self, in_msg: &[u8]) -> Result<RotelMessage> {
        let captures = self
            .regex
            .captures(in_msg)
            .ok_or_else(|| eyre!("Failed to match pattern to input.".to_string()))?;
        let m = &captures
            .name("value")
            .ok_or_else(|| eyre!("Failed to match value".to_string()))?;
        let value = std::str::from_utf8(m.as_bytes())
            .map_err(|e| eyre!(format!("Message is not UTF-8: {}", e)))?;
        let parser = self.parser;
        parser(value)
    }
}

fn parse_volume(value: &str) -> Result<RotelMessage> {
    let volume = value.parse()?;
    Ok(RotelMessage::Volume(volume))
}

fn parse_mute(value: &str) -> Result<RotelMessage> {
    let muted = match value {
        "on" => true,
        "off" => false,
        _ => {
            return Err(eyre!(format!(
                "Failed to parse mute. Invalid value: {}",
                value
            )))
        }
    };
    Ok(RotelMessage::Mute(muted))
}

pub static ROTEL_MSG_READERS: Lazy<Vec<RotelMessageParser>> = Lazy::new(|| {
    vec![
        RotelMessageParser::new(VOLUME_PATTERN, parse_volume).unwrap(),
        RotelMessageParser::new(MUTE_PATTERN, parse_mute).unwrap(),
    ]
});

static ROTEL_MSG_REGEX: Lazy<RegexSet> = Lazy::new(|| {
    let patterns: Vec<&str> = ROTEL_MSG_READERS.iter().map(|x| x.regex.as_str()).collect();
    let mut builder = RegexSetBuilder::new(&patterns);
    builder.case_insensitive(true);
    builder.unicode(false);
    builder.build().unwrap()
});
