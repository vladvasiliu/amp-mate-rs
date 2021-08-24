use clap::{App, Arg, ArgGroup, ArgMatches};

pub fn get_config() -> ArgMatches {
    App::new("Amp Mate")
        .arg(
            Arg::new("amp")
                .short('a')
                .long("amp")
                .value_name("AMP_ADDR")
                .about("Amplifier address")
                .takes_value(true)
                .required(true),
        )
        .subcommand(
            App::new("follow")
                .about("Follow amp output")
                .arg(
                    Arg::new("format-volume")
                        .about("Volume format")
                        .value_name("FORMAT")
                        .long("format-volume")
                        .short('v')
                        .required(false)
                        .takes_value(true)
                        .default_value("{value}"),
                )
                .arg(
                    Arg::new("format-mute")
                        .about("Mute format")
                        .value_name("FORMAT")
                        .long("format-mute")
                        .short('m')
                        .required(false)
                        .takes_value(true)
                        .default_value("{value}"),
                ),
        )
        .subcommand(
            App::new("one-shot")
                .about("Send a command and quit")
                .group(
                    ArgGroup::new("action")
                        .multiple(false)
                        .required(true)
                        .args(&["volume", "mute"]),
                )
                .arg(
                    Arg::new("volume")
                        .short('v')
                        .long("volume")
                        .value_name("VOLUME")
                        .about("Set volume")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("mute")
                        .short('m')
                        .long("mute")
                        .value_name("MUTE")
                        .about("Set mute")
                        .takes_value(true),
                ),
        )
        .get_matches()
}
