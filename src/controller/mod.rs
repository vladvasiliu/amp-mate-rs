pub(crate) mod protocol;

use self::protocol::{RotelCommand, RotelResponse};
use color_eyre::eyre::Result;
use log::{debug, info, warn};
use std::convert::TryFrom;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{task, try_join};

pub struct RotelController {
    address: String,
}

impl RotelController {
    pub fn new(address: String) -> Self {
        Self { address }
    }

    /// Sends a command to the amp and returns an error if sending failed.
    pub async fn one_shot(&self, command: RotelCommand) -> Result<()> {
        let mut stream = TcpStream::connect(&self.address).await?;
        info!("Connected to {}", stream.peer_addr()?);
        stream.set_nodelay(true)?;
        stream.write_all(command.to_rotel().as_bytes()).await?;
        Ok(())
    }

    pub async fn run(
        &self,
        command_channel: Receiver<RotelCommand>,
        response_channel: Sender<RotelResponse>,
    ) -> Result<()> {
        let stream = TcpStream::connect(&self.address).await?;
        info!("Connected to {}", stream.peer_addr()?);
        stream.set_nodelay(true)?;
        let (read_half, write_half) = stream.into_split();
        let reader_task = task::spawn(async { read_from_amp(read_half, response_channel).await });
        let writer_task =
            task::spawn(async move { write_to_amp(write_half, command_channel).await });
        try_join!(reader_task, writer_task).map(|_| Ok(()))?
    }
}

/// Reads messages from the amplifier and sends them through a channel
///
/// Every message should end with a `$`.
/// If a read returns with 0 bytes it means the connection is lost, therefore return an error.
/// Messages must be UTF-8.
async fn read_from_amp(
    amp_stream: impl AsyncRead + Unpin,
    channel: Sender<RotelResponse>,
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
    mut amp_stream: impl AsyncWrite + Unpin,
    mut channel: Receiver<RotelCommand>,
) -> Result<()> {
    debug!("Started writer");
    while let Some(msg) = channel.recv().await {
        amp_stream.write_all(msg.to_rotel().as_bytes()).await?;
    }
    Ok(())
}
