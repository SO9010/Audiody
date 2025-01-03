use super::{save::settings, setup::music_dir};
use crate::api::types::Book;
use std::{fs, path::PathBuf};

pub fn get_saved_book(book_title: String) -> Result<Option<Book>, Box<dyn std::error::Error>> {
    let music_dir: PathBuf = music_dir()?;
    let mut book: Option<Book> = None;

    for entry in fs::read_dir(music_dir)? {
        let entry = entry?;
        let mut chapter_urls = Vec::new();
        let mut image_url = Vec::new();
        let mut settings: settings = settings {
            book_url: "".to_string(),
            current_chapter: None,
            current_chapter_time: None,
        };

        if entry
            .path()
            .display()
            .to_string()
            .contains(book_title.as_str())
        {
            for item in fs::read_dir(entry.path())? {
                let item = item?;
                if item.path().is_file() {
                    let file_name = item.path().display().to_string();

                    if file_name.contains(".webp") {
                        log::info!("Found: {}", file_name);
                        image_url.push(file_name);
                    } else if file_name.contains(".mp3") {
                        if !file_name.contains("_NA") {
                            chapter_urls.push(file_name);
                        }
                    } else if file_name.contains("settings.json") {
                        settings =
                            serde_json::from_str(&fs::read_to_string(file_name).unwrap()).unwrap();
                    }
                }
            }

            // Ensure image_url is valid and has at least one entry
            let image_url = image_url
                .first()
                .cloned()
                .unwrap_or_else(|| "default_image_url".to_string()); // Replace with a fallback value if necessary

            chapter_urls.sort_by(|a, b| {
                let a_num = extract_number(a).unwrap_or(0);
                let b_num = extract_number(b).unwrap_or(0);
                a_num.cmp(&b_num)
            });
            
            log::info!("Chapter URLs: {}", chapter_urls.join(", "));
            book = Some(Book {
                saved: true,
                title: entry.file_name().into_string().unwrap_or_default(),
                chapter_urls,
                chapter_durations: vec![], // Update with appropriate data
                chapter_reader: vec![],    // Update with appropriate data
                description: "".to_string(),
                author: "".to_string(),
                url: settings.book_url.to_string(),
                image_URL: image_url,
            });
            return Ok(book);
        }
    }
    Ok(book)
}

pub fn get_saved_books() -> Result<Vec<Book>, Box<dyn std::error::Error>> {
    let mut books: Vec<Book> = Vec::new();

    let music_dir = music_dir()?;

    for entry in fs::read_dir(music_dir)? {
        let entry = entry?;
        let mut chapter_urls = Vec::new();
        let mut image_url = Vec::new();
        let mut settings: settings = settings {
            book_url: "".to_string(),
            current_chapter: None,
            current_chapter_time: None,
        };

        for item in fs::read_dir(entry.path())? {
            let item = item?;
            if item.path().is_file() {
                let file_name = item.path().display().to_string();

                if file_name.contains(".webp") {
                    log::info!("Found: {}", file_name);
                    image_url.push(file_name);
                } else if file_name.contains(".mp3") {
                    chapter_urls.push(file_name);
                } else if file_name.contains("settings.json") {
                    settings =
                        serde_json::from_str(&fs::read_to_string(file_name).unwrap()).unwrap();
                }
            }
        }

        // Ensure image_url is valid and has at least one entry
        let image_url = image_url
            .first()
            .cloned()
            .unwrap_or_else(|| "default_image_url".to_string()); // Replace with a fallback value if necessary

        books.push(Book {
            saved: true,
            title: entry.file_name().into_string().unwrap_or_default(),
            chapter_urls,
            chapter_durations: vec![], // Update with appropriate data
            chapter_reader: vec![],    // Update with appropriate data
            description: "".to_string(),
            author: "".to_string(),
            url: settings.book_url.to_string(),
            image_URL: image_url,
        });
    }

    Ok(books)
}

pub fn check_book_chapter_url(chapt: u32, title: String) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let music_dir: PathBuf = music_dir()?.join(title);
    let mut chapter_urls = Vec::new();

    for item in fs::read_dir(music_dir.as_path())? {
        let item = item?;
        if item.path().is_file() {
            let file_name = item.path().display().to_string();

            if file_name.contains(".mp3") && !file_name.contains("_NA") {
                if file_name.contains("chapter") && file_name.contains(chapt.to_string().as_str()) {
                    return Ok(Some(music_dir.join(file_name)));
                } else if (extract_number(&file_name).unwrap() - 1) == chapt {
                    return Ok(Some(music_dir.join(file_name)));
                    
                }
                chapter_urls.push(file_name);
            }
        }
    }
    Ok(None)
}

pub fn extract_number(filename: &str) -> Option<u32> {
    // Split the filename by hyphens and extract the numeric part
    filename
        .split('-')
        .nth(1) // Get the second segment (e.g., "999")
        .and_then(|s| s.parse::<u32>().ok())
}
