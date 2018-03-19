//! A utility that stitches 4 images together
//!
//! run the binary:
//! $ stitcher --using foo
//!
//! Assuming you had the following files:
//! - foo-tl.png
//! - foo-tr.png
//! - foo-bl.png
//! - foo-br.png
//!
//! This will output a png called foo-out.png which contains all four of the
//! above files stitched together in a 2x2 layout
//!
//! building and running with info log levels:
//! STITCHER_LOG='info' stitcher --base foo
#[macro_use]
extern crate failure;
extern crate image;
#[macro_use]
extern crate log;

use std::io::Error as IoError;
use std::fs::File;

use image::{DynamicImage, GenericImage, ImageBuffer};

/// A specialized `Result` type for the `Stitcher` crate.
pub type Result<T> = ::std::result::Result<T, StitcherError>;

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

fn check_size(img: &DynamicImage, expected_width: u32, expected_height: u32) -> Result<()> {
    let (width, height) = img.dimensions();

    if width != expected_width || height != expected_height {
        return Err(StitcherError::SizeMismatch);
    }

    Ok(())
}

fn copy_into(
    img: &mut image::RgbaImage,
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

/// Stitch together four images that meet the following requirements:
///
/// 1. Must be in png format
/// 2. Have names ending in '-tl', '-tr', '-bl' and '-br' for top-left, top-right,
///    bottom-left and bottom-right respectively
/// 3. All images must have the same dimensions
///
/// # Examples
///
/// ```
/// use stitcher::stitch;
///
/// stitch("artwork")?;
/// ```
///
/// Assuming that the image files: 'artwork-tl.png', 'artwork-tl.png',
/// 'artwork-tl.png' and 'artwork-tl.png' exist, the function will combine them
/// into a single file called 'artwork-out.png' which is saved in the same location
/// as the input files
pub fn stitch(using: &str) -> Result<()> {
    info!("stitch:{}", using);

    let filename_tl = format!("{}-tl.png", using);
    let filename_tr = format!("{}-tr.png", using);
    let filename_bl = format!("{}-bl.png", using);
    let filename_br = format!("{}-br.png", using);
    let filename_output = format!("{}-out.png", using);

    stitch_images(
        &filename_tl,
        &filename_tr,
        &filename_bl,
        &filename_br,
        &filename_output,
    )
}

pub fn stitch_images(tl: &str, tr: &str, bl: &str, br: &str, out: &str) -> Result<()> {
    info!("stitch_images: {} {} {} {} -> {}", tl, tr, bl, br, out);

    let img_tl = image::open(tl)?;
    let img_tr = image::open(tr)?;
    let img_bl = image::open(bl)?;
    let img_br = image::open(br)?;

    // all images should have the same dimensions
    let (width, height) = img_tl.dimensions();
    check_size(&img_tr, width, height)?;
    check_size(&img_bl, width, height)?;
    check_size(&img_br, width, height)?;

    // Construct a new ImageBuffer for all 4 images
    let mut img = ImageBuffer::new(width * 2, height * 2);

    copy_into(&mut img, &img_tl, 0, 0, width, height)?;
    copy_into(&mut img, &img_tr, width, 0, width, height)?;
    copy_into(&mut img, &img_bl, 0, height, width, height)?;
    copy_into(&mut img, &img_br, width, height, width, height)?;

    let ref mut fout = File::create(out)?;
    image::ImageRgba8(img).save(fout, image::PNG)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
