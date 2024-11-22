use crate::api::types::*;
use scraper::{Html, Selector};

pub struct LibriVoxClient {
    client: reqwest::Client,
    base_url: String,
}

impl LibriVoxClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://librivox.app/".to_string(),
        }
    }
}

// https://librivox.app/search.jsp?search=marxism

impl LibriVoxClient {
    pub async fn search(&self, query: String) -> Result<Vec<Book>, AudiodyError> {
        let url = reqwest::Url::parse(&(self.base_url.clone() + format!("search.jsp?search={}", query).as_str()))
            .map_err(|e| return AudiodyError::ParseError(e.to_string()))?;
        
        log::info!("URL: {}", url.to_string());
        let response = reqwest::get(url).await?;
        let document = Html::parse_document(&response.text().await?);

        // Update selector for book container
        let book_selector = Selector::parse("div[style*='min-height:120px']").unwrap();
        
        Ok(document.select(&book_selector)
        .map(|book_element| {
            // Get cover image and URL
            let cover_selector = Selector::parse("img[style*='float:left']").unwrap();
            let cover_url = book_element
                .select(&cover_selector)
                .next()
                .and_then(|img| img.value().attr("src"))
                .unwrap_or("")
                .to_string();

            // Get title and book URL
            let title_selector = Selector::parse("h3.booklist-title a").unwrap();
            let title_element = book_element.select(&title_selector).next();
            let title = title_element
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
            let book_url = title_element
                .and_then(|e| e.value().attr("href"))
                .unwrap_or("")
                .to_string();

            // Get author info
            let author_selector = Selector::parse("h4.booklist-author a").unwrap();
            let author = book_element
                .select(&author_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            // Get reader info
            let reader_selector = Selector::parse("h4.reader a").unwrap();
            let readers = book_element
                .select(&reader_selector)
                .next()
                .map(|e| e.text().collect::<String>().trim().to_string());

            // Get description - text after the star ratings
            let description = book_element
                .text()
                .collect::<String>()
                .lines()
                .filter(|line| line.trim().len() > 0)
                .last()
                .unwrap_or("")
                .trim()
                .to_string();

            Book {
                saved: false,
                title,
                author,
                url: book_url,
                image_URL: cover_url,
            }
        })
        .collect())
    }
}
