// Copyright ⓒ 2018 Inderjit Gill
// Licensed under the MIT license
// (see LICENSE-MIT or <http://opensource.org/licenses/MIT>) All files in the project carrying such
// notice may not be copied, modified, or distributed except according to those terms.

//! 'stitcher' is a utility that stitches 4 images together
//!
//!
//! ## Running the binary:
//!
//! ### Conventional usage
//!
//! ```no_run
//! $ stitcher -x 2 -y 2 -i a.png b.png c.png d.png -o result.png
//! ```
//!
//! This will output a png called result.png which contains all four of the
//! above files stitched together in a 2x2 layout
//!
//! ### Debug usage
//!
//! To show different log levels, set the STITCHER_LOG environment variable to one of the following:
//!
//! - trace
//! - debug
//! - info
//! - warn
//! - error
//!
//! $ STITCHER_LOG='trace' stitcher -x 2 -y 2 -i a.png b.png c.png d.png -o result.png
//!
//! ## License
//!
//! `stitcher` is licensed under the MIT license. Please read the [LICENSE-MIT](LICENSE-MIT) file in
//! this repository for more information.

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate failure;
extern crate image;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use std::io::Error as IoError;
use std::fs::File;

use image::{DynamicImage, GenericImage, ImageBuffer, ImageResult, RgbaImage};

/// A specialized `Result` type for the `Stitcher` crate.
pub type Result<T> = ::std::result::Result<T, StitcherError>;

fn main() {
    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

fn run() -> Result<()> {
    env_logger::init_from_env("STITCHER_LOG");

    let matches = App::new("Stitcher")
        .version("0.2.0")
        .author("Inderjit Gill <email@indy.io>")
        .about("Stitches images of the same dimension together")
        .arg(
            Arg::with_name("width")
                .short("x")
                .long("width")
                .help("The number of images along the x-axis")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("height")
                .short("y")
                .long("height")
                .help("The number of images along the y-axis")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("image")
                .short("i")
                .long("image")
                .help("an image to use")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Sets the output image")
                .takes_value(true),
        )
        .get_matches();

    // get command line arguments

    let filenames = matches
        .values_of("image")
        .map(|vals| vals.collect::<Vec<_>>())
        .unwrap_or(Vec::new());

    let x = value_t!(matches, "width", u32).unwrap_or(1);
    let y = value_t!(matches, "height", u32).unwrap_or(1);

    let output;
    if let Some(o) = matches.value_of("output") {
        output = o;
    } else {
        return Err(StitcherError::CommandLineParsingError);
    }

    // sanity check command line arguments

    if filenames.len() as u32 != x * y {
        error!(
            "width:{} x height:{} mismatch with given images:{}, expected:{}",
            x,
            y,
            filenames.len(),
            x * y
        );
        return Err(StitcherError::CommandLineParsingError);
    };

    // check image dimensions

    let images: Vec<ImageResult<DynamicImage>> =
        filenames.into_iter().map(|f| image::open(f)).collect();

    let (width, height) = size_of_first(&images)?;
    check_dimensions(&images, width, height)?;

    // create the combined image

    let mut img: RgbaImage = ImageBuffer::new(width * x, height * y);
    let mut iter = images.iter();

    for yy in 0..y {
        for xx in 0..x {
            if let Some(block) = iter.next() {
                if let &Ok(ref block_) = block {
                    copy_into(&mut img, &block_, xx * width, yy * height, width, height)?;
                }
            }
        }
    }

    // save to disk

    let ref mut fout = File::create(output)?;
    image::ImageRgba8(img).save(fout, image::PNG)?;

    Ok(())
}

fn size_of_first(images: &Vec<ImageResult<DynamicImage>>) -> Result<(u32, u32)> {
    // get the size of the first image
    //
    let first = images.into_iter().nth(0).unwrap();

    if let &Ok(ref first_image) = first {
        Ok(first_image.dimensions())
    } else {
        Err(StitcherError::SizeMismatch)
    }
}

fn check_dimensions(
    images: &Vec<ImageResult<DynamicImage>>,
    width: u32,
    height: u32,
) -> Result<()> {
    // compare the rest of the images with the size of the first image
    //
    let res = images
        .into_iter()
        .skip(1)
        .all(|ref image| is_same_size(&image, width, height));

    if res == true {
        Ok(())
    } else {
        Err(StitcherError::SizeMismatch)
    }
}

fn is_same_size(
    image: &std::result::Result<image::DynamicImage, image::ImageError>,
    width: u32,
    height: u32,
) -> bool {
    if let &Ok(ref img) = image {
        let (width_, height_) = img.dimensions();
        return width_ == width && height_ == height;
    }

    false
}

fn copy_into(
    img: &mut RgbaImage,
    src: &DynamicImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<()> {
    let mut sub = img.sub_image(x, y, width, height);
    sub.copy_from(src, 0, 0);

    Ok(())
}

#[derive(Debug, Fail)]
pub enum StitcherError {
    #[fail(display = "an io error occurred")] Io(#[cause] IoError),

    #[fail(display = "an image error occurred")] ImageError(#[cause] image::ImageError),

    #[fail(display = "Command line parsing")] CommandLineParsingError,

    #[fail(display = "Image size mismatch")] SizeMismatch,

    /// This allows you to produce any `failure::Error` within closures used by
    /// the skeleton crate. No errors of this kind will ever be produced by the
    /// crate itself.
    #[fail(display = "{}", inner)]
    Custom {
        /// The actual error that occurred.
        inner: failure::Error,
    },
}

impl From<IoError> for StitcherError {
    fn from(e: IoError) -> StitcherError {
        StitcherError::Io(e)
    }
}

impl From<image::ImageError> for StitcherError {
    fn from(e: image::ImageError) -> StitcherError {
        StitcherError::ImageError(e)
    }
}
