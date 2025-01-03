use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, thread, vec};

use crate::api::types::*;
use rusty_ytdl::search::{SearchResult, YouTube};
use rusty_ytdl::Video;
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct YouTubeClient {
    youtube: YouTube,
    base_url: String,
}

impl Default for YouTubeClient {
    fn default() -> Self {
        Self::new()
    }
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
        let results = self
            .youtube
            .search(query + " audiobook", None)
            .await
            .unwrap();

        let books: Vec<Book> = results
            .iter()
            .filter_map(|result| {
                match result {
                    SearchResult::Video(video) => Some(Book {
                        title: video.title.clone(),
                        author: video.channel.name.clone(),
                        image_URL: video.thumbnails.first().unwrap().url.clone(),
                        url: video.url.clone(),
                        description: video.description.clone(),
                        saved: false,
                        chapter_urls: vec![video.url.clone()],
                        chapter_durations: vec![video.duration.to_string()],
                        chapter_reader: vec![video.channel.name.clone()],
                    }),
                    _ => None, // Skip playlists and channels
                }
            })
            .collect();

        Ok(books)
    }
    pub async fn get_book(&self, url: String) -> Result<Book, ureq::Error> {
        let video = Video::new(url.clone()).unwrap();

        let video_info: rusty_ytdl::VideoInfo = video.get_info().await.unwrap();

        Ok(Book {
            saved: false,
            title: video_info.video_details.title,
            chapter_urls: video_info
                .video_details
                .chapters
                .iter()
                .map(|capt| video_info.video_details.video_url.to_string())
                .collect(),
            chapter_durations: video_info
                .video_details
                .chapters
                .iter()
                .map(|chapter| chapter.start_time.to_string())
                .collect(),
            chapter_reader: vec![video_info.video_details.owner_channel_name.clone()],
            description: video_info.video_details.description,
            author: video_info.video_details.owner_channel_name.clone(),
            url,
            image_URL: video_info.video_details.thumbnails[0].url.clone(),
        })
    }

    // We should have two one for specific chapter and one for the whole book
    pub fn get_chapter(
        &self,
        url: String,
        path: String,
        chapter: i32,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let video = Video::new(url.clone()).unwrap();

        let video_info: rusty_ytdl::VideoInfo =
            Runtime::new().unwrap().block_on(video.get_info()).unwrap();

        let libraries_dir = PathBuf::from("libs");
        let youtube = libraries_dir.join("yt-dlp");

        // For some reason it takes so much longer to download a single chapter

    
        let mut chapter_urls = vec![];
        for item in fs::read_dir(path.clone())? {
            let item = item?;
            if item.path().is_file() {
                let file_name = item.path().display().to_string();

                if file_name.contains(".mp3") {
                    chapter_urls.push(file_name);
                }
            }
        }
        
        if chapter_urls.len() == 0 {
            let _ = Command::new(youtube)
            .args([
                "--concurrent-fragments",
                "5",               // Number of fragments to download concurrently
                "--extract-audio", // Extract audio only
                "--audio-format",
                "mp3",              // Set audio format to mp3
                "--write-auto-sub", // Download auto-generated subtitles
                "--sub-lang",
                "en",                   // Set subtitle language to English
                "--split-chapters",     // Split the video into chapters
                "--restrict-filenames", // Ensure filenames are safe and consistent
                "--write-thumbnail",    // Download the thumbnail
                "-P",
                &path, // Output directory
                "-o",
                "chapter_%(section_number)s.%(ext)s", // Output filename template
                &url,                                 // Video URL
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .expect("failed to execute process");
            
            for item in fs::read_dir(path.clone())? {
                let item = item?;
                if item.path().is_file() {
                    let file_name = item.path().display().to_string();
    
                    if file_name.contains(".mp3") {
                        chapter_urls.push(file_name);
                    }
                }
            }
        }
        Ok(PathBuf::from(path).join(chapter_urls.get(chapter as usize).unwrap()))
    }
}
