mod protocol;

use self::protocol::{RotelCommand, RotelQuery, RotelResponse, ToRotel, Volume};
use color_eyre::eyre::Result;
use log::{debug, info, warn};
use std::convert::TryFrom;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct RotelController {
    address: String,
    command_channel: Receiver<Box<dyn ToRotel>>,
    response_channel: Sender<RotelResponse>,
}

impl RotelController {
    pub fn new(
        address: String,
        command_channel: Receiver<Box<dyn ToRotel>>,
        response_channel: Sender<RotelResponse>,
    ) -> Self {
        Self {
            address,
            command_channel,
            response_channel,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let stream = TcpStream::connect(&self.address).await?;
        info!("Connected to {}", stream.peer_addr()?);
        stream.set_nodelay(true)?;
        let (read_half, mut write_half) = stream.into_split();
        tokio::join!(
            read_from_amp(read_half, &self.response_channel),
            write_to_amp(&mut write_half, &mut self.command_channel)
        );
        Ok(())
    }
}

/// Reads messages from the amplifier and sends them through a channel
///
/// Every message should end with a `$`.
/// If a read returns with 0 bytes it means the connection is lost, therefore return an error.
/// Messages must be UTF-8.
async fn read_from_amp(
    amp_stream: impl AsyncRead + Unpin,
    channel: &Sender<RotelResponse>,
) -> Result<()> {
    let buffered_reader = BufReader::new(amp_stream);
    let mut command_reader = buffered_reader.split(b'$');
    debug!("Started reader");
    while let Some(segment) = command_reader.next_segment().await?.as_deref() {
        // match protocol::decode_amp_message(&segment) {
        match RotelResponse::try_from(segment) {
            Err(e) => warn!("Received unexpected message from amp: {}", e),
            Ok(response) => channel.send(response).await?,
        }
    }
    Ok(())
}

/// Reads messages from a channel and sends them to the amp.
async fn write_to_amp(
    amp_stream: &mut (impl AsyncWrite + Unpin),
    channel: &mut Receiver<Box<dyn ToRotel>>,
) -> Result<()> {
    debug!("Started writer");
    while let Some(msg) = channel.recv().await.as_deref() {
        amp_stream.write_all(msg.to_rotel()).await?;
    }
    Ok(())
}
