use std::fs::{self, File};
use std::path::{Path, PathBuf};
use image::ImageFormat;
use ureq;

use crate::api::webimage::url_to_buffer;
use crate::api::yt::YouTubeClient;

use super::setup::music_dir;

/// Book, Chapter, URL
pub fn download_audio(
    book: &str,
    chapt: i32,
    url: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // TODO: WE NEED TO ADD A METADATA FILE WHICH STATES THINGS SUCH AS PLAY THROUGH, DESCRIPTION AND CHAPTERS and all their URLS!

    // Send the GET request to the URL
    let audio_path = music_dir().unwrap().as_path().join(book);
    fs::create_dir_all(&audio_path)?;
    // Open the output file to write the audio content
    let output_file = audio_path.join(format!("chapter_{}.mp3", chapt));

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

                let parent = Path::new(url).parent().unwrap();
                parent.join("__ia_thumb.jpg");

           
                let mut image_file = File::create(output_file.clone().with_extension("webp"))?;
                // TODO FINISH THIS
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
}
pub fn save_progress(book: &str, chapt: i32, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
