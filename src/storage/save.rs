use image::io::Reader;
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::PathBuf;
use ureq;
use webp::Encoder;

use crate::api::yt::YouTubeClient;

use super::saved::extract_number;
use super::setup::music_dir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct settings {
    pub book_url: String,
    pub current_chapter: Option<i32>,
    pub current_chapter_time: Option<f64>,
}

impl settings {
    pub fn new() -> Self {
        settings {
            book_url: "".to_string(),
            current_chapter: None,
            current_chapter_time: None,
        }
    }

    pub fn save(&self, file_path: &PathBuf) -> io::Result<()> {
        let mut file = File::create(file_path)?;
        let mut writer = BufWriter::new(&mut file);
        serde_json::to_writer(&mut writer, self)?;
        Ok(())
    }

    pub fn load(file_path: &PathBuf) -> io::Result<Self> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let settings = serde_json::from_reader(reader)?;
        Ok(settings)
    }
}

/// Book, Chapter, URL
pub fn download_audio(
    book: &str,
    chapt: i32,
    url: &str,
    book_url: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Send the GET request to the URL
    let audio_path = music_dir().unwrap().as_path().join(book);
    fs::create_dir_all(&audio_path)?;
    // Open the output file to write the audio content
    let mut output_file = audio_path.clone().join(format!("chapter_{}.mp3", chapt));
    if !audio_path.join("settings.json").exists() {
        save_progress(book, None, book_url, None)?;
    }
    for item in fs::read_dir(audio_path.clone())? {
        let item = item?;
        if item.path().is_file() {
            let file_name = item.path().display().to_string();

            if file_name.contains(".mp3") && !file_name.contains("_NA") && ((extract_number(PathBuf::from(audio_path.clone()).join(file_name.clone()).display().to_string().as_str()).unwrap()-1) as i32 == chapt) {
                output_file = PathBuf::from(audio_path.clone()).join(file_name.clone());
            } 
        }
    }

    if !output_file.exists() {
        if url.contains("youtube") {
            let yt = YouTubeClient::new();

            return Ok(yt.get_chapter(
                url.to_string(),
                audio_path.to_str().unwrap().to_string(),
                chapt,
            )?);
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

                let encoder = Encoder::from_rgba(
                    rgba_image.as_raw(),
                    rgba_image.width(),
                    rgba_image.height(),
                );
                let webp_data = encoder.encode(75.0); // Adjust quality if needed

                let webp_path = output_file.with_extension("webp");
                let output_file = File::create(&webp_path)?;
                let mut writer = BufWriter::new(output_file);
                fs::remove_file(jpg_path)?;
                writer.write_all(&webp_data)?;
            }

            log::info!(
                "Downloading book {book} to: {} from {url}",
                output_file.display()
            );

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
pub fn save_progress(
    book: &str,
    chapt: Option<i32>,
    url: &str,
    chapter_play_time: Option<f64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let audio_path = music_dir().unwrap().as_path().join(book);
    let settings_file = audio_path.clone().join("settings.json");
    let mut settings = settings::new();
    settings = settings {
        book_url: url.to_string(),
        current_chapter: chapt,
        current_chapter_time: chapter_play_time,
    };

    settings.save(&settings_file)?;

    Ok(())
}

pub fn get_progress(book: &str) -> Result<settings, Box<dyn std::error::Error>> {
    let audio_path = music_dir().unwrap().as_path().join(book);
    let settings_file = audio_path.clone().join("settings.json");
    let settings_str = fs::read_to_string(settings_file)?;
    Ok(serde_json::from_str(&settings_str)?)
}
