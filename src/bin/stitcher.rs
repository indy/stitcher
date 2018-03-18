extern crate clap;
extern crate env_logger;
extern crate stitcher;

use clap::{App, Arg};

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

// building and running with info log levels
// STITCHER_LOG='info' cargo run --bin stitcher -- --base something

fn run() -> stitcher::Result<()> {
    env_logger::init_from_env("STITCHER_LOG");

    let matches = App::new("Stitcher")
        .version("0.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("Stitches 4 images into 1")
        .arg(
            Arg::with_name("base")
                .short("b")
                .long("base")
                .value_name("FILE")
                .help("base name of input files")
                .required(true),
        )
        .get_matches();

    match matches.value_of("base") {
        Some(base) => stitcher::stitch(base),
        None => Err(stitcher::StitcherError::CommandLineParsingError),
    }
}
