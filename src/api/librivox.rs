use std::vec;

use crate::api::types::*;
use serde_json::json;
use ureq::{Agent, Request, Response};
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
    pub fn search(&self, query: String) -> Result<Vec<Book>, ureq::Error> {
        let url = format!("{}search.jsp?search={}", self.base_url, json!(query));
        let body: String = ureq::get(&url)
            .call()?
            .into_string()?;


        log::info!("Librivox search: {}", url);
        let document = Html::parse_document(&body);

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
            .map(|src| {
                if src.starts_with("http") {
                    src.to_string()
                } else {
                    format!("{}{}", self.base_url.trim_end_matches('/'), src)
                }
            })
            .unwrap_or("".to_string());
        

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
                description: "".to_string(),
                author,
                url: book_url,
                image_URL: cover_url,
                chapter_urls: vec![],
                chapter_durations: vec![],
                chapter_reader: vec![],
            }
        })
        .collect())
    }
    
    pub fn get_book(&self, url: String) -> Result<Book, ureq::Error> {
        let mut url = url;
        url.remove(0);
        let url = format!("{}{}", self.base_url, url);

        let body = ureq::get(&url)
            .call().unwrap()
            .into_string()?;

        log::info!("Calling: {}", url);
        let document = Html::parse_document(&body);
                
        // Extract title with fallback
        let title_selector = Selector::parse("h1.book-title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        // Extract author with fallback
        let author_selector = Selector::parse("h2.author a").unwrap();
        let author = document
            .select(&author_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        // Extract cover image URL with default placeholder
        let image_selector = Selector::parse("div.cover-image img").unwrap();
        let image_url = document
            .select(&image_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|src| src.to_string())
            .unwrap_or_else(|| "default_cover.jpg".to_string());

        // Extract description with meaningful fallback
        let desc_selector = Selector::parse("div#description p[itemprop='description']").unwrap();
        let description = document
            .select(&desc_selector)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "No description available".to_string());

        // Extract chapters with validation
        let chapters_selector = Selector::parse("table.table.table-striped tr").unwrap();
        let chapters: Vec<(String, String, String)> = document
            .select(&chapters_selector)
            .filter_map(|row| {
                let link = row.select(&Selector::parse("td a").unwrap())
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .map(|s| s.to_string());
                
                let duration = row.select(&Selector::parse("td:nth-child(2)").unwrap())
                    .next()
                    .map(|td| td.text().collect::<String>().trim().to_string());

                let reader = row.select(&Selector::parse("td:nth-child(3)").unwrap())
                    .next()
                    .map(|td| td.text().collect::<String>().trim().to_string());

                match (link, duration, reader) {
                    (Some(l), Some(d), Some(r)) => Some((l, d, r)),
                    _ => None
                }
            })
            .collect();

        if chapters.is_empty() {
            log::warn!("No chapters found for book: {}", title);
        }

        Ok(Book {
            saved: false,
            title,
            description,
            author,
            url,
            image_URL: image_url,
            chapter_urls: chapters.iter().map(|(link, _, _)| link.clone()).collect(),
            chapter_durations: chapters.iter().map(|(_, duration, _)| duration.clone()).collect(),
            chapter_reader: chapters.iter().map(|(_, _, reader)| reader.clone()).collect(),
        })
    }
}
