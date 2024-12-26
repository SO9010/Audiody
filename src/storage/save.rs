use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use image::io::Reader;
use image::DynamicImage;
use serde::{Serialize, Deserialize};
use ureq;
use webp::Encoder;

use crate::api::yt::YouTubeClient;

use super::setup::music_dir;

#[derive(Serialize, Deserialize, Debug)]
pub struct settings {
    pub book_url: String
}

/// Book, Chapter, URL
pub fn download_audio(
    book: &str,
    chapt: i32,
    url: &str,
    book_url: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // TODO: WE NEED TO ADD A METADATA FILE WHICH STATES THINGS SUCH AS PLAY THROUGH, DESCRIPTION AND CHAPTERS and all their URLS!

    // Send the GET request to the URL
    let audio_path = music_dir().unwrap().as_path().join(book);
    fs::create_dir_all(&audio_path)?;
    // Open the output file to write the audio content
    let output_file = audio_path.clone().join(format!("chapter_{}.mp3", chapt));
    let settings_file = audio_path.clone().join("settings.json");
    let mut settings = File::create(settings_file)?;
    settings.write_all(serde_json::to_string(
        &settings {
            book_url: book_url.to_string(),
        }
    ).unwrap().as_bytes());

    if !output_file.exists() {
        if url.contains("youtube") {
            let yt = YouTubeClient::new();

            yt.get_chapter(
                url.to_string(),
                output_file.to_str().unwrap().to_string(),
                chapt,
            )?;
        } else {

            // Make this less hardcoded and more flexible to get iamges!
            if url.contains("archive.org") {
                let pos = url.rfind("/");
                let (img, _) = url.split_at(pos.unwrap());
                let img = img.to_owned() + "/__ia_thumb.jpg";
        
                log::info!("Downloading image {img}");
        
                let response = ureq::get(&img).call()?;
                if response.status() != 200 {
                    return Err(format!("Failed to download file: {}", response.status()).into());
                }
        
                // Save the downloaded .jpg file
                let mut p_osstr = output_file.clone().as_os_str().to_owned();
                p_osstr.push(".jpg");
                let jpg_path = std::path::Path::new(&p_osstr);
        
                let mut jpg_file = File::create(jpg_path)?;
                let mut reader = response.into_reader();
                io::copy(&mut reader, &mut jpg_file)?;
        
                // Convert the .jpg file to .webp
                log::info!("Converting image to WEBP format");
                let img = Reader::open(jpg_path)?.decode()?;
                let rgba_image = match img {
                    DynamicImage::ImageRgba8(img) => img,
                    other => other.to_rgba8(),
                };
        
                let encoder = Encoder::from_rgba(rgba_image.as_raw(), rgba_image.width(), rgba_image.height());
                let webp_data = encoder.encode(75.0); // Adjust quality if needed
        
                let webp_path = output_file.with_extension("webp");
                let output_file = File::create(&webp_path)?;
                let mut writer = BufWriter::new(output_file);
                fs::remove_file(jpg_path);
                writer.write_all(&webp_data)?;
            }   
            
            log::info!("Downloading book {book} to: {} from {url}", output_file.display());
                
            let response = ureq::get(url).call()?;

            if response.status() != 200 {
                return Err(format!("Failed to download file: {}", response.status()).into());
            }

            let mut file = File::create(output_file.clone())?;
            let mut reader = response.into_reader();
            std::io::copy(&mut reader, &mut file)?;
        }
    }

    Ok(output_file)
}pub fn save_progress(book: &str, chapt: i32, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
