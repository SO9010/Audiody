use std::fs::{self, File};
use std::path::PathBuf;
use dirs::audio_dir;
use tokio::runtime::Runtime;
use ureq;

use crate::api::yt::YouTubeClient;

/// Book, Chapter, URL 
pub fn download_audio(book: &str, chapt: i32, url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Send the GET request to the URL
    let book = book
    .to_lowercase()
    .replace(" ", "_")
    .replace("\"", "")
    .replace("\'", "")
    .replace(",", "")
    .replace(".", "")
    .replace(":", "")
    .replace(";", "")
    .replace("!", "")
    .replace("?", "")
    .replace("'", "")
    .replace("-", "_")
    .replace("(", "")
    .replace(")", "");

    log::info!("Downoading: {}", book);
    let audio_path = audio_dir().ok_or("Failed to get audio directory")?.join("books").join(book);
    fs::create_dir_all(&audio_path)?;
    // Open the output file to write the audio content
    let output_file = audio_path.join(format!("chapter_{}.mp3", chapt));
    
    if !output_file.exists() {
        if url.contains("youtube") {
            let yt = YouTubeClient::new();
            
            Runtime::new().unwrap().block_on(yt.get_chapter(url.to_string(), output_file.to_str().unwrap().to_string(), chapt))?;
        } else {
            log::info!("Downloading: {}", url);
            let response = ureq::get(url).call()?;
    
            if response.status() != 200 {
                return Err(format!("Failed to download file: {}", response.status()).into());
            }
            
            let mut file = File::create(audio_path.clone())?;
            let mut reader = response.into_reader();
            std::io::copy(&mut reader, &mut file)?;
        }
    }

    Ok(output_file)
}