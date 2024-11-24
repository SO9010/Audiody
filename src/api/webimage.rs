use reqwest;
use slint::{Image, SharedPixelBuffer, Rgba8Pixel};
use std::io::Cursor;
use image::io::Reader as ImageReader;

use super::types::AudiodyError;

pub async fn url_to_buffer(url: &str) -> Result<Image, AudiodyError> {
    // Fetch the image data from URL using blocking client
    let response = reqwest::blocking::get(url)?;
    let image_data = response.bytes()?;
    
    // Convert to image buffer
    let img = ImageReader::new(Cursor::new(image_data))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    
    // Convert to RGBA8
    let rgba_img = img.into_rgba8();
    let width = rgba_img.width() as u32;
    let height = rgba_img.height() as u32;
    
    // Create SharedPixelBuffer from raw pixels
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(
        width,
        height,
    );
    
    // Fill the pixel buffer with image data
    pixel_buffer.make_mut_bytes().copy_from_slice(&rgba_img.into_raw());
    
    Ok(Image::from_rgba8(pixel_buffer))
}
