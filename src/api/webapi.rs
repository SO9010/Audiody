use super::{librivox::LibriVoxClient, types::Book, webimage::url_to_buffer, yt::YouTubeClient};

// Auto set language to English which is recorded_langage=1
/*
https://librivox.org/
http://www.loyalbooks.com/
http://thoughtaudio.com/
https://www.storynory.com/
http://www.loyalbooks.com/book/a-midsummer-nights-dream-by-william-shakespeare
https://stephenking.com/works/audiobook/index.html
https://archive.org/
https://freeclassicaudiobooks.com/
https://www.learnoutloud.com/Free-Audiobooks
*/

#[derive(Debug, Clone)]
pub struct WebApiClient {
    youtube_client: YouTubeClient,
    libri_client: LibriVoxClient,
}

impl WebApiClient {
    pub fn new() -> Self {
        Self {
            youtube_client: YouTubeClient::new(),
            libri_client: LibriVoxClient::new(),
        }
    }
}

// https://librivox.app/search.jsp?search=marxism

impl WebApiClient {
    
    pub async fn search(&self, query: String) -> Result<Vec<Vec<Book>>, ureq::Error> {
        // Librivox search
        let libri_books = self.libri_client.search(query.to_string()).unwrap();

        // Youtube search
        let yt_books = self.youtube_client.search(query.to_string()).await?;
        let books = vec![libri_books, yt_books];
        Ok(books)
    }

    pub async fn get_book(&self, url: String) -> Result<Book, ureq::Error> {
        // Librivox search
        log::info!("Getting book: {}", url);
        if url.contains("librivox") {
            let libri_book = self.libri_client.get_book(url).unwrap();
            return Ok(libri_book);
        } else if url.contains("youtube.com") {
            let yt_book = self.youtube_client.get_book(url).await?;
            return Ok(yt_book);
        } else {
            Ok(Book {
                title: "".to_string(),
                author: "".to_string(),
                image_URL: "".to_string(),
                url: "".to_string(),
                description: "".to_string(),
                saved: false,
                chapter_urls: vec![],
                chapter_durations: vec![],
                chapter_reader: vec![],
            })
        }
    }
}