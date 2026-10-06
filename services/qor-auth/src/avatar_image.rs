//! Cleaning an uploaded avatar (ADR-079 decision 3).
//!
//! What a person uploads is never stored. It is decoded, turned the right way up (a photo's orientation), cropped to a
//! square from its centre, resized to 256 x 256, and encoded again: a still image as PNG, an animated GIF as a GIF.
//! Nothing of the original file survives but its pixels, so a photo's GPS position and every other piece of metadata is
//! gone, and a file crafted to attack a decoder reaches only this one, written in Rust, under memory limits.

use std::io::Cursor;

use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::imageops::FilterType;
use image::{
    AnimationDecoder, DynamicImage, Frame, ImageDecoder, ImageFormat, ImageReader, Limits,
    RgbaImage,
};

/// The largest upload accepted.
pub const MAX_UPLOAD: usize = 4 * 1024 * 1024;
/// The largest avatar stored (an animated one; a still is far smaller).
pub const MAX_STORED: usize = 2 * 1024 * 1024;
/// The most frames an animated avatar keeps.
pub const MAX_FRAMES: usize = 120;
/// Every avatar is this many pixels square.
pub const SIDE: u32 = 256;
/// The largest image dimension decoded at all.
const MAX_DIMENSION: u32 = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    TooLarge,
    Unsupported,
    Unreadable,
    TooManyFrames,
}

impl Refusal {
    /// The sentence the person reads.
    pub fn message(&self) -> &'static str {
        match self {
            Refusal::TooLarge => {
                "That file is too large. Use an image under 4 MB, or a GIF that stays under 2 MB."
            }
            Refusal::Unsupported => "Use a PNG, JPEG, WebP or GIF image.",
            Refusal::Unreadable => "That image could not be read. Try another file.",
            Refusal::TooManyFrames => "That GIF has too many frames. Use one with at most 120.",
        }
    }
}

/// A cleaned avatar, ready to store, and for an animated one its first frame as a still PNG.
pub struct Clean {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub still: Option<Vec<u8>>,
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(256 * 1024 * 1024);
    limits
}

/// The centre square of an image, resized to `SIDE`.
fn square(image: &DynamicImage, filter: FilterType) -> RgbaImage {
    let side = image.width().min(image.height());
    let x = (image.width() - side) / 2;
    let y = (image.height() - side) / 2;
    image
        .crop_imm(x, y, side, side)
        .resize_exact(SIDE, SIDE, filter)
        .to_rgba8()
}

/// Decode, clean and re-encode an upload.
pub fn clean(input: &[u8]) -> Result<Clean, Refusal> {
    if input.len() > MAX_UPLOAD {
        return Err(Refusal::TooLarge);
    }
    let format = image::guess_format(input).map_err(|_| Refusal::Unsupported)?;
    match format {
        ImageFormat::Gif => clean_gif(input),
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => clean_still(input, format),
        _ => Err(Refusal::Unsupported),
    }
}

fn clean_still(input: &[u8], format: ImageFormat) -> Result<Clean, Refusal> {
    let mut reader = ImageReader::with_format(Cursor::new(input), format);
    reader.limits(limits());
    let mut decoder = reader.into_decoder().map_err(|_| Refusal::Unreadable)?;
    let orientation = decoder.orientation().map_err(|_| Refusal::Unreadable)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(|_| Refusal::Unreadable)?;
    image.apply_orientation(orientation);
    let square = square(&image, FilterType::Lanczos3);
    let mut bytes = Vec::new();
    DynamicImage::ImageRgba8(square)
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| Refusal::Unreadable)?;
    Ok(Clean {
        bytes,
        content_type: "image/png",
        still: None,
    })
}

fn clean_gif(input: &[u8]) -> Result<Clean, Refusal> {
    let mut decoder = GifDecoder::new(Cursor::new(input)).map_err(|_| Refusal::Unreadable)?;
    decoder
        .set_limits(limits())
        .map_err(|_| Refusal::TooLarge)?;
    let mut frames = Vec::new();
    for frame in decoder.into_frames() {
        let frame = frame.map_err(|_| Refusal::Unreadable)?;
        if frames.len() == MAX_FRAMES {
            return Err(Refusal::TooManyFrames);
        }
        let delay = frame.delay();
        let pixels = DynamicImage::ImageRgba8(frame.into_buffer());
        frames.push(Frame::from_parts(
            square(&pixels, FilterType::Triangle),
            0,
            0,
            delay,
        ));
    }
    match frames.len() {
        0 => Err(Refusal::Unreadable),
        // A GIF with one frame is a still image.
        1 => {
            let still = frames
                .pop()
                .map(Frame::into_buffer)
                .ok_or(Refusal::Unreadable)?;
            let mut bytes = Vec::new();
            DynamicImage::ImageRgba8(still)
                .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
                .map_err(|_| Refusal::Unreadable)?;
            Ok(Clean {
                bytes,
                content_type: "image/png",
                still: None,
            })
        }
        _ => {
            // The first frame alone, for people who asked for less motion (ADR-079 decision 6).
            let mut still = Vec::new();
            DynamicImage::ImageRgba8(frames[0].buffer().clone())
                .write_to(&mut Cursor::new(&mut still), ImageFormat::Png)
                .map_err(|_| Refusal::Unreadable)?;
            let mut bytes = Vec::new();
            {
                let mut encoder = GifEncoder::new_with_speed(&mut bytes, 10);
                encoder
                    .set_repeat(Repeat::Infinite)
                    .map_err(|_| Refusal::Unreadable)?;
                encoder
                    .encode_frames(frames)
                    .map_err(|_| Refusal::Unreadable)?;
            }
            if bytes.len() > MAX_STORED {
                return Err(Refusal::TooLarge);
            }
            Ok(Clean {
                bytes,
                content_type: "image/gif",
                still: Some(still),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Delay, Rgba};

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = RgbaImage::from_fn(width, height, |x, y| {
            Rgba([(x % 256) as u8, (y % 256) as u8, 90, 255])
        });
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        bytes
    }

    fn gif(frames: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = GifEncoder::new(&mut bytes);
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for i in 0..frames {
                let image =
                    RgbaImage::from_pixel(40, 30, Rgba([(i * 20 % 256) as u8, 10, 10, 255]));
                encoder
                    .encode_frame(Frame::from_parts(
                        image,
                        0,
                        0,
                        Delay::from_numer_denom_ms(100, 1),
                    ))
                    .unwrap();
            }
        }
        bytes
    }

    fn dimensions(bytes: &[u8]) -> (u32, u32) {
        let image = image::load_from_memory(bytes).unwrap();
        (image.width(), image.height())
    }

    #[test]
    fn a_still_image_becomes_a_256_square_png() {
        let clean = clean(&png(640, 360)).expect("cleaned");
        assert_eq!(clean.content_type, "image/png");
        assert_eq!(dimensions(&clean.bytes), (256, 256));
    }

    #[test]
    fn metadata_does_not_survive() {
        // A PNG carrying a text chunk, as a photo carries EXIF: the cleaned file has none of it.
        let marker = b"GPSLatitude 51.5074 secret";
        let mut data = b"Comment ".to_vec();
        data.extend_from_slice(marker);
        let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
        let mut typed = b"tEXt".to_vec();
        typed.extend_from_slice(&data);
        chunk.extend_from_slice(&typed);
        chunk.extend_from_slice(&crc32(&typed).to_be_bytes());
        let mut bytes = png(64, 64);
        // After the 8-byte signature and the 25-byte IHDR chunk.
        bytes.splice(33..33, chunk);
        assert!(image::load_from_memory(&bytes).is_ok(), "still a valid PNG");
        assert!(
            bytes.windows(marker.len()).any(|w| w == marker),
            "the upload carries it"
        );
        let clean = clean(&bytes).expect("cleaned");
        assert!(
            !clean.bytes.windows(marker.len()).any(|w| w == marker),
            "the stored avatar does not"
        );
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for byte in data {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = if crc & 1 == 1 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    #[test]
    fn an_animated_gif_stays_animated_at_256() {
        let clean = clean(&gif(5)).expect("cleaned");
        assert_eq!(clean.content_type, "image/gif");
        let frames = GifDecoder::new(Cursor::new(&clean.bytes))
            .unwrap()
            .into_frames()
            .collect_frames()
            .unwrap();
        assert_eq!(frames.len(), 5);
        assert_eq!(frames[0].buffer().dimensions(), (256, 256));
        let still = clean.still.expect("a still first frame");
        assert_eq!(image::guess_format(&still).unwrap(), ImageFormat::Png);
        assert_eq!(image::load_from_memory(&still).unwrap().width(), 256);
    }

    #[test]
    fn a_one_frame_gif_is_a_still() {
        assert_eq!(clean(&gif(1)).unwrap().content_type, "image/png");
    }

    #[test]
    fn what_is_refused() {
        assert_eq!(
            clean(&gif(MAX_FRAMES + 1)).err(),
            Some(Refusal::TooManyFrames)
        );
        assert_eq!(
            clean(b"not an image at all").err(),
            Some(Refusal::Unsupported)
        );
        assert_eq!(
            clean(&vec![0u8; MAX_UPLOAD + 1]).err(),
            Some(Refusal::TooLarge)
        );
        let mut broken = png(64, 64);
        broken.truncate(60);
        assert_eq!(clean(&broken).err(), Some(Refusal::Unreadable));
    }
}
