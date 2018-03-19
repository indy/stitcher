#[macro_use]
extern crate log;
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

fn run() -> stitcher::Result<()> {
    env_logger::init_from_env("STITCHER_LOG");

    let matches = App::new("Stitcher")
        .version("0.1.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("Stitches 4 images into 1")
        .arg(
            Arg::with_name("using")
                .short("u")
                .long("using")
                .help("use naming convention to determine input files")
        )
        .arg(
            Arg::with_name("top-left")
                .short("l")
                .long("top-left")
                .help("Sets the top left image")
                .takes_value(true))
        .arg(
            Arg::with_name("top-right")
                .short("t")
                .long("top-right")
                .help("Sets the top right image")
                .takes_value(true))
        .arg(
            Arg::with_name("bottom-left")
                .short("b")
                .long("bottom-left")
                .help("Sets the bottom left image")
                .takes_value(true))
        .arg(
            Arg::with_name("bottom-right")
                .short("r")
                .long("bottom-right")
                .help("Sets the bottom right image")
                .takes_value(true))
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Sets the output image")
                .takes_value(true))
        .get_matches();

    if let Some(using) = matches.value_of("using") {
        return stitcher::stitch(using);
    }

    // check if we have _all_ of the images specified, return an error otherwise
    //
    let tl = matches.value_of("top-left");
    let tr = matches.value_of("top-right");
    let bl = matches.value_of("bottom-left");
    let br = matches.value_of("bottom-right");
    let out = matches.value_of("output");

    if let Some(tl) = tl {
        if let Some(tr) = tr {
            if let Some(bl) = bl {
                if let Some(br) = br {
                    if let Some(out) = out {
                        return stitcher::stitch_images(tl, tr, bl, br, out);
                    }
                }
            }
        }
    }

    // tl is empty, if any of the others aren't then print a warning message
    if tr.is_some() || bl.is_some() || br.is_some() || out.is_some() {
        error!("either specify a common 'use' value or explicitly specify all four input images and an output filename");
        return Err(stitcher::StitcherError::CommandLineParsingError);
    }

    Ok(())
}
