use std::path::{Path, PathBuf};
use std::{fs, vec};
use std::process::{ Command, Stdio };

use crate::api::types::*;
use rusty_ytdl::{Chapter, Video};
use rusty_ytdl::search::{SearchResult, YouTube};

#[derive(Debug, Clone)]
pub struct YouTubeClient {
    youtube: YouTube,
    base_url: String,
}

impl YouTubeClient {
    pub fn new() -> Self {
        Self {
            youtube: YouTube::new().unwrap(),
            base_url: "https://www.youtube.com/".to_string(),
        }
    }
}

// Find a way to improve the image quality!
  impl YouTubeClient {
    pub async fn search(&self, query: String) -> Result<Vec<Book>, ureq::Error> {
        let results = self.youtube.search(query + " audiobook", None).await.unwrap();
        
        let books: Vec<Book> = results.iter()
            .filter_map(|result| {
                match result {
                    SearchResult::Video(video) => Some(Book {
                        title: video.title.clone(),
                        author: video.channel.name.clone(),
                        image_URL: video.thumbnails.get(0).unwrap().url.clone(),
                        url: video.url.clone(),
                        description: video.description.clone(),
                        saved: false,
                        chapter_urls: vec![video.url.clone()],
                        chapter_durations: vec![video.duration.to_string()],
                        chapter_reader: vec![video.channel.name.clone()],
                    }),
                    _ => None // Skip playlists and channels
                }
            })
            .collect();
    
        Ok(books)
    }
    pub async fn get_book(&self, url: String) -> Result<Book, ureq::Error> {
        let video = Video::new(url.clone()).unwrap();

        let video_info = video.get_info().await.unwrap();

        Ok(Book {
            saved: false,
            title: video_info.video_details.title,
            chapter_urls: vec![video_info.video_details.video_url],
            chapter_durations: video_info.video_details.chapters.iter().map(|chapter| chapter.start_time.to_string()).collect(),
            chapter_reader: vec![video_info.video_details.owner_channel_name.clone()],
            description: video_info.video_details.description,
            author: video_info.video_details.owner_channel_name.clone(),
            url,
            image_URL: video_info.video_details.thumbnails[0].url.clone(),
        })
    }

    pub async fn get_chapter(&self, url: String, path: String, chapter: i32) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let libraries_dir = PathBuf::from("libs");
        let youtube = libraries_dir.join("yt-dlp");
        // NEED TO ADD PROGRESS BAR
        Command::new(youtube)
            .args(
                &[
                    "--extract-audio",
                    "--audio-format",
                    "mp3",
                    "-o",
                    &path.clone(),
                    &url.clone(),
                ]
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        Ok(PathBuf::from(path))
    }}
