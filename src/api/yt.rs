use std::vec;
use std::error::Error;

use crate::api::types::*;
use rusty_ytdl::Video;
use rusty_ytdl::search::{SearchResult, YouTube};

pub struct YouTubeClient {
    base_url: String,
}

impl YouTubeClient {
    pub fn new() -> Self {
        Self {
            // https://pipedapi.kavin.rocks/opensearch/suggestions?query=Audiobook
            base_url: "https://www.youtube.com/".to_string(),
        }
    }
}

// https://librivox.app/search.jsp?search=marxism

  impl YouTubeClient {
    pub async fn search(&self, query: String) -> Result<Vec<Book>, Box<dyn Error>> {
      let youtube = YouTube::new().unwrap();
      let results = youtube.search(query+" audiobook", None).await?;
      
      let books = results.iter()
          .filter_map(|result| {
              match result {
                  SearchResult::Video(video) => Some(Book {
                      title: video.title.clone(),
                      author: video.channel.name.clone(),
                      image_URL: format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", video.id),
                      url: format!("/watch?v={}", video.id),
                      description: video.description.clone(),
                      saved: false,
                      chapter_urls: vec![format!("/watch?v={}", video.id)],
                      chapter_durations: vec![video.duration.to_string()],
                      chapter_reader: vec![video.channel.name.clone()],
                  }),
                  _ => None // Skip playlists and channels
              }
          })
          .collect();
  
      Ok(books)
  }
}
