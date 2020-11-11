use clap::{App, Arg, ArgMatches};

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
        .get_matches()
}
