mod config;
mod controller;
mod polybar;

use crate::config::get_config;
use crate::controller::protocol::{Change, Direction, RotelCommand, Volume, StateToggle};
use crate::controller::RotelController;
use color_eyre::eyre::{Result, Report};
use log::{info, LevelFilter};
use tokio::sync::mpsc::channel;
use tokio::{select, task};
use crate::polybar::PolybarOutput;
use tokio::signal::unix::{signal, SignalKind};

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
    } else {
        info!("Running in follow mode");
        let (command_channel_tx, command_channel_rx) = channel(8);
        let (response_channel_tx, mut response_channel_rx) = channel(8);
        let mut polybar_output = PolybarOutput::new(command_channel_tx.clone(), response_channel_rx);

        let rotel_handle =
            task::spawn(async move { rotel.run(command_channel_rx, response_channel_tx).await });
        let polybar_handle = task::spawn(async move { polybar_output.run().await });
        let signal_listener = tokio::spawn(
            async move {
                loop {
                    let mut usr1_stream = signal(SignalKind::user_defined1())?;
                    let mut usr2_stream = signal(SignalKind::user_defined2())?;
                    let command = select! {
                        _ = usr1_stream.recv() => { RotelCommand::Set(Change::Volume(Volume::Direction(Direction::Up)))}
                        _ = usr2_stream.recv() => { RotelCommand::Set(Change::Volume(Volume::Direction(Direction::Down)))}
                    };
                    command_channel_tx.send(command).await?;
                }
                Ok::<(), Report>(())
            });

        select! {
            _ = rotel_handle => {}
            _ = polybar_handle => {}
            _ = signal_listener => {}
        }
    }
    Ok(())
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
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}
