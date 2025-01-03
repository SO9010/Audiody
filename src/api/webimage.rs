use image::{io::Reader as ImageReader, open};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::io::Cursor;

pub async fn url_to_buffer(url: String) -> Result<Image, Box<dyn std::error::Error>> {
    log::info!("Converting: {url}");
    let mut url = url;
    if url.contains("lh3.googleusercontent.com") {
        url.push('0');
    }
    let rgba_img;
    if !url.contains(".webp") {
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;
        log::info!("Getting image from {}", url);

        // Get bytes from response
        let buffer = response.bytes().await?;
        // Convert to image buffer with proper error handling
        let img = ImageReader::new(Cursor::new(buffer))
            .with_guessed_format()?
            .decode()?;
        // Convert to RGBA8
        rgba_img = img.into_rgba8();
    } else {
        rgba_img = open(&url).unwrap().into_rgba8();
    }

    let width = rgba_img.width();
    let height = rgba_img.height();

    // Create SharedPixelBuffer from raw pixels
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);

    // Fill the pixel buffer with image data
    pixel_buffer
        .make_mut_bytes()
        .copy_from_slice(&rgba_img.into_raw());

    Ok(Image::from_rgba8(pixel_buffer))
}
