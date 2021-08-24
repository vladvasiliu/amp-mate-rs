mod config;
mod controller;
mod polybar;

use crate::config::get_config;
use crate::controller::protocol::{Change, Direction, RotelCommand, StateToggle, Volume};
use crate::controller::RotelController;
use crate::polybar::{OutputFormatter, PolybarOutput};
use color_eyre::eyre::{eyre, Report, Result};
use log::{error, info, LevelFilter};
use std::str::FromStr;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc::channel;
use tokio::{select, task};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logger(LevelFilter::Debug)?;

    let config = get_config();

    let address: String = config.value_of_t_or_exit::<String>("amp");
    let rotel = RotelController::new(address);

    match config.subcommand() {
        Some(("one-shot", matches)) => {
            if config.is_present("volume") {
                let volume = matches.value_of_t_or_exit::<Volume>("volume");
                rotel
                    .one_shot(RotelCommand::Set(Change::Volume(volume)))
                    .await?;
            } else if config.is_present("mute") {
                let mute: StateToggle = matches.value_of_t_or_exit("mute");
                rotel
                    .one_shot(RotelCommand::Set(Change::Mute(mute)))
                    .await?;
            }
        }
        Some(("follow", follow_matches)) => {
            info!("Running in follow mode");
            let volume_formatter =
                OutputFormatter::from_str(follow_matches.value_of("format-volume").unwrap())
                    .map_err(|e| eyre!("Invalid volume format: {}", e))?;
            let mute_formatter =
                OutputFormatter::from_str(follow_matches.value_of("format-mute").unwrap())
                    .map_err(|e| eyre!("Invalid mute format: {}", e))?;
            let (command_channel_tx, command_channel_rx) = channel(8);
            let (response_channel_tx, response_channel_rx) = channel(8);
            let mut polybar_output = PolybarOutput::new(
                command_channel_tx.clone(),
                response_channel_rx,
                volume_formatter,
                mute_formatter,
            );

            let rotel_handle =
                task::spawn(
                    async move { rotel.run(command_channel_rx, response_channel_tx).await },
                );
            let polybar_handle = task::spawn(async move { polybar_output.run().await });
            let signal_listener = tokio::spawn(async move {
                let mut usr1_stream = signal(SignalKind::user_defined1())?;
                let mut usr2_stream = signal(SignalKind::user_defined2())?;
                loop {
                    let command = select! {
                        _ = usr1_stream.recv() => { RotelCommand::Set(Change::Volume(Volume::Direction(Direction::Up)))}
                        _ = usr2_stream.recv() => { RotelCommand::Set(Change::Volume(Volume::Direction(Direction::Down)))}
                    };
                    command_channel_tx.send(command).await?;
                }
                // Can't define the return type for an async block
                // https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
                #[allow(unreachable_code)]
                Ok::<(), Report>(())
            });

            select! {
                rotel_result = rotel_handle => {
                    if let Err(err) = rotel_result.unwrap() {
                        error!("Rotel controller error: {}", err);
                    }
                }
                _ = polybar_handle => {}
                _ = signal_listener => {}
            }
        }
        _ => error!("Missing command"),
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
