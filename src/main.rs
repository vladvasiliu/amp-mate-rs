mod config;
mod controller;

use crate::config::get_config;
use crate::controller::protocol::{Change, RotelCommand, Volume, StateToggle};
use crate::controller::RotelController;
use color_eyre::eyre::{eyre, Result};
use log::{error, info, LevelFilter};
use std::convert::TryFrom;
use tokio::sync::mpsc::channel;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logger(LevelFilter::Debug)?;

    let config = get_config();

    let address: String = config.value_of_t_or_exit::<String>("amp");
    let rotel = RotelController::new(address);

    if config.is_present("volume") {
        let volume = config.value_of_t_or_exit::<Volume>("volume");
        rotel.one_shot(RotelCommand::Set(Change::Volume(volume))).await?;
    } else if config.is_present("mute") {
        let mute: StateToggle = config.value_of_t_or_exit("mute");
        rotel.one_shot(RotelCommand::Set(Change::Mute(mute))).await?;
    }

    Ok(())

    // let (command_channel_tx, command_channel_rx) = channel(8);
    // let (response_channel_tx, mut response_channel_rx) = channel(8);
    // let run_handle =
    //     task::spawn(async move { rotel.run(command_channel_rx, response_channel_tx).await });
    // command_channel_tx
    //     .send(RotelCommand::Set(Change::Volume(
    //         Volume::try_from(10).unwrap(),
    //     )))
    //     .await?;
    // while let Some(response) = response_channel_rx.recv().await {
    //     info!("{:?}", response);
    // }
    // run_handle.await?
}

fn setup_logger(level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[ {} ][ {:5} ][ {:15} ] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
