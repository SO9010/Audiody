use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudiodyError {
    #[error("Request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

pub struct SearchQuery {
    pub title: Option<String>,
    pub author: Option<String>,
    pub reader: Option<String>,
    pub keywords: Option<String>,
    pub genre_id: Option<u32>,
    pub sort_order: Option<String>,
    pub page: Option<u32>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Books {
    pub books: Vec<Book>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Book {
    pub saved : bool,
    pub title: String,
    pub chapter_urls: Vec<String>,
    pub chapter_durations: Vec<String>,
    pub chapter_reader: Vec<String>,
    pub description: String,
    pub author: String,
    pub url: String,
    pub image_URL: String,
    // pub saved in drive
}
