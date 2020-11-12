mod protocol;

use super::EntityController;
use crate::controller::rotel::protocol::{RotelCommand, RotelResponse, Volume};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use log::{info, warn};
use protocol::RotelQuery;
use std::convert::TryFrom;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub struct RotelController {
    pub address: String,
}

#[async_trait]
impl EntityController for RotelController {
    async fn run(&self) -> Result<()> {
        let stream = TcpStream::connect(&self.address).await?;
        info!("Connected to {}", stream.peer_addr()?);
        stream.set_nodelay(true)?;

        let (mut read_half, mut write_half) = stream.into_split();
        write_half.write_all(b"rs232_update_on!").await?;
        write_half.write_all(RotelQuery::Volume.into()).await?;
        let volume = Volume::try_from(24)?;
        write_half
            .write_all(RotelCommand::Volume(volume).build_command().as_bytes())
            .await?;
        read_from_amp(&mut read_half).await?;
        Ok(())
    }
}

/// Reads messages from the amplifier, waiting indefinitely, and sending them through a channel
///
/// Every message should end with a `$`.
/// If a read returns with 0 bytes it means the connection is lost, therefore return an error.
/// Messages are expected to be UTF-8.
async fn read_from_amp(amp_stream: &mut (impl AsyncRead + Unpin)) -> Result<()> {
    let buffered_reader = BufReader::new(amp_stream);
    let mut command_reader = buffered_reader.split(b'$');
    while let Some(segment) = command_reader.next_segment().await?.as_deref() {
        // match protocol::decode_amp_message(&segment) {
        match RotelResponse::try_from(segment) {
            Err(e) => warn!("{}", e),
            Ok(msg) => info!("Read msg: {:#?}", msg),
        }
    }
    Ok(())
}
