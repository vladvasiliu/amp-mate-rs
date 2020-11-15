mod config;
mod controller;

use crate::config::get_config;
use color_eyre::eyre::Result;
use controller::RotelController;
use log::LevelFilter;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    setup_logger(LevelFilter::Debug)?;

    let config = get_config();

    let address: String = config.value_of_t_or_exit::<String>("amp");
    let (command_channel_tx, command_channel_rx) = channel(8);
    let (response_channel_tx, reponse_channel_rx) = channel(8);
    let mut rotel = RotelController::new(address, command_channel_rx, response_channel_tx);
    rotel.run().await?;

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
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
