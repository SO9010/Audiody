use slint::{Image, SharedPixelBuffer, Rgba8Pixel};
use std::io::Cursor;
use image::io::Reader as ImageReader;
use ureq::{Agent, Request, Response};

use super::types::AudiodyError;

pub fn url_to_buffer(url: String) -> Result<Image, Box<dyn std::error::Error>> {
    // Fetch the image data from URL using blocking client
    let mut url = url;
    if (url.find("lh3.googleusercontent.com").is_some()) {
        url.push('0');
    }
    let response = ureq::get(&url).call()?;
    
    // Read response body into a Vec<u8>
    let mut buffer = Vec::new();
    response.into_reader().read_to_end(&mut buffer)?;

    // Convert to image buffer with proper error handling
    let img = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()?
        .decode()?;

    // Convert to RGBA8
    let rgba_img = img.into_rgba8();
    let width = rgba_img.width() as u32;
    let height = rgba_img.height() as u32;

    // Create SharedPixelBuffer from raw pixels
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    
    // Fill the pixel buffer with image data
    pixel_buffer.make_mut_bytes().copy_from_slice(&rgba_img.into_raw());
    
    Ok(Image::from_rgba8(pixel_buffer))
}