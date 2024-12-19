use audio::audios::AudioService;
use slint::{ComponentHandle, Model, VecModel};
use std::thread::{self, Thread};
use std::{path::PathBuf, rc::Rc, vec};

pub mod api;
pub mod audio;
pub mod storage;

use api::{webapi::WebApiClient, webimage::url_to_buffer};
use storage::save::download_audio;
use storage::saved::get_saved_books;
use storage::saved::get_saved_book;
use storage::setup::music_dir;
use tokio::runtime::Runtime; // 0.3.5

use yt_dlp::Youtube;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

slint::include_modules!();

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    env_logger::init();
    let state = init();
    let main_window = state.main_window.clone_strong();

    #[cfg(target_os = "android")]
    STATE.with(|ui| *ui.borrow_mut() = Some(state));
    tokio::task::block_in_place(|| main_window.run().unwrap());
}

fn init() -> State {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    // YT-DLP download should only be ran once
    // TODO: Make this a background task
    // Downloading binaries
    thread::spawn(|| {
        // change this to be in the home folder, or the normal place to install scripts!
        let executables_dir = PathBuf::from("libs");
        let output_dir = PathBuf::from("output");
        log::info!("Downloading binaries");
        Runtime::new()
            .unwrap()
            .block_on(Youtube::with_new_binaries(executables_dir, output_dir))
            .unwrap();
    });

    /* Update the downloads
    let libraries_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");

    let libraries = Libraries::new(youtube, ffmpeg);
    let fetcher = Youtube::new(libraries, output_dir).unwrap();

    Runtime::new().unwrap().block_on(fetcher.update_downloader()).unwrap();
    */

    let main_window = AppWindow::new().unwrap();
    let audio_state = main_window.global::<AudioState>();
    let audio_service = AudioService::new();

    let webapi_client = WebApiClient::new();
    // Implement populating the books one by one do then it looks more dynamic and doesnt just wait ages
    // At somepoint we want to implement multi threading so that* the requests dont get int the way of the UI
    let previous_views = Rc::new(std::cell::RefCell::new(vec![0]));

    let main_window_weak = main_window.as_weak();
    let previous_views_clone = previous_views.clone();

    // Set up the download folder
    music_dir();

    // Get saved books:
    let saved_books = get_saved_books().unwrap();
    let saved_books_converted: Vec<BookItem> = saved_books
        .into_iter()
        .map(|book| BookItem {
            title: book.title.into(),
            author: book.author.into(),
            description: book.description.clone().into(),
            book_url: book.url.into(),
            saved: book.saved,
            image: Runtime::new()
                .unwrap()
                .block_on(url_to_buffer(book.image_URL))
                .unwrap(),
            chapter_urls: slint::ModelRc::new(slint::VecModel::from(
                book.chapter_urls
                    .into_iter()
                    .map(|url| url.into())
                    .collect::<Vec<slint::SharedString>>(),
            )),
            chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])),
            chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])),
        })
        .collect();
    main_window
        .global::<AudioState>()
        .set_home_page_books(slint::ModelRc::new(slint::VecModel::from(
            saved_books_converted,
        )));

    audio_state.on_go_to_previous_page(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            if let Some(prev_view) = previous_views_clone.borrow_mut().pop() {
                if prev_view == 0 {
                    main_window
                        .global::<AudioState>()
                        .set_page_name("Audiody".into());
                }
                main_window
                    .global::<AudioState>()
                    .set_current_view(prev_view.into())
            }
        }
    });

    let previous_views_clone = previous_views.clone();
    audio_state.on_add_previous_page(move |page| {
        previous_views_clone.borrow_mut().push(page);
    });

    let main_window_weak = main_window.as_weak();
    audio_state.on_on_search_clicked(move |query| {
        // We also need to implement caching!
        let books = Runtime::new()
            .unwrap()
            .block_on(webapi_client.clone().search(query.to_string()))
            .unwrap();
        let libri_book_items: Vec<BookItem> = books[0]
            .clone()
            .into_iter()
            .map(|book| BookItem {
                title: book.title.into(),
                author: book.author.into(),
                description: book.description.clone().into(),
                book_url: book.url.into(),
                saved: book.saved,
                image: Runtime::new()
                    .unwrap()
                    .block_on(url_to_buffer(book.image_URL))
                    .unwrap(),
                chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])),
                chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])),
                chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])),
            })
            .collect();
        let libri_book_model = slint::ModelRc::new(slint::VecModel::from(libri_book_items));

        let yt_book_items: Vec<BookItem> = books[1]
            .clone()
            .into_iter()
            .map(|book| BookItem {
                title: book.title.into(),
                author: book.author.into(),
                description: book.description.clone().into(),
                book_url: book.url.into(),
                saved: book.saved,
                image: Runtime::new()
                    .unwrap()
                    .block_on(url_to_buffer(book.image_URL))
                    .unwrap(),
                chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])),
                chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])),
                chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])),
            })
            .collect();
        let yt_book_model = slint::ModelRc::new(slint::VecModel::from(yt_book_items));

        if let Some(main_window) = main_window_weak.upgrade() {
            main_window
                .global::<AudioState>()
                .set_search_libi(libri_book_model);
            main_window
                .global::<AudioState>()
                .set_search_yt(yt_book_model);
        }
    });

    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_toggle_pause(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            if main_window.global::<AudioState>().get_paused() {
                audio_service_clone.play();
            } else {
                audio_service_clone.pause();
            }
        }
    });

    let audio_service_clone = audio_service.clone();
    audio_state.on_skip_backward(move || {
        audio_service_clone.seek_relative(-10);
    });

    let audio_service_clone = audio_service.clone();
    audio_state.on_skip_forward(move || {
        audio_service_clone.seek_relative(10);
    });

    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_change_speed(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            audio_service_clone.set_speed(main_window.global::<AudioState>().get_speed());
        }
    });

    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_chapter_download_and_play(move |book, chapter, URL| {
        if let Some(main_window) = main_window_weak.upgrade() {
            match download_audio(&book, chapter, &URL) {
                Ok(path_buf) => {
                    if let Some(path_str) = path_buf.to_str() {
                        log::info!("Downloaded audio path: {}", path_str);
                        audio_service_clone.start(path_str.to_string());
                        audio_service_clone.play();
                        main_window.global::<AudioState>().set_paused(false);
                        // TODO: Optimise this so that it doesnt refresh the whole thing
                        main_window.global::<AudioState>().get_home_page_books();
                    } else {
                        log::info!("Error: Path contains invalid UTF-8");
                    }
                }
                Err(e) => {
                    log::info!("Error downloading audio: {}", e);
                }
            }
        }
    });

    let main_window_weak = main_window.as_weak();
    audio_state.on_chapter_download(move |book, chapter, URL| {
        if let Some(main_window) = main_window_weak.upgrade() {

        match download_audio(&book, chapter, &URL) {
            Ok(path_buf) => {
                if let Some(path_str) = path_buf.to_str() {
                    log::info!("Downloaded audio path: {}", path_str);
                    main_window.global::<AudioState>().get_home_page_books().as_any().downcast_ref::<VecModel<BookItem>>().unwrap().push(main_window.global::<AudioState>().get_book_view());
                } else {
                    log::info!("Error: Path contains invalid UTF-8");
                }
            }
            Err(e) => {
                log::info!("Error downloading audio: {}", e);
            }
        }}
    });

    let main_window_weak = main_window.as_weak();
    let webapi_client = WebApiClient::new();
    audio_state.on_on_book_view(move |bookURL, bookTitle| {
        let main_window_weak = main_window_weak.clone();
        let book: api::types::Book;
        if let Ok(Some(book_thing)) = get_saved_book(bookTitle.to_string()) {
            book = book_thing;
        } else {
            // We also need to implement caching!
            let main_window_weak = main_window_weak.clone();
            book = Runtime::new()
                .unwrap()
                .block_on(webapi_client.clone().get_book(bookURL.to_string()))
                .unwrap();
        }

        let book_item = BookItem {
            title: book.title.into(),
            author: book.author.into(),
            description: book.description.clone().into(),
            book_url: book.url.into(),
            saved: book.saved,
            image: Runtime::new()
                .unwrap()
                .block_on(url_to_buffer(book.image_URL))
                .unwrap(),
            // Find a better way of doing this
            chapter_urls: slint::ModelRc::new(slint::VecModel::from(
                book.chapter_urls
                    .into_iter()
                    .map(|url| url.into())
                    .collect::<Vec<slint::SharedString>>(),
            )),

            chapter_durations: slint::ModelRc::new(slint::VecModel::from(
                book.chapter_durations
                    .into_iter()
                    .map(|dur| dur.into())
                    .collect::<Vec<slint::SharedString>>(),
            )),

            chapter_reader: slint::ModelRc::new(slint::VecModel::from(
                book.chapter_reader
                    .into_iter()
                    .map(|reader| reader.into())
                    .collect::<Vec<slint::SharedString>>(),
            )),
        };

        if let Some(main_window) = main_window_weak.upgrade() {
            main_window.global::<AudioState>().set_book_view(book_item);
        }
    });

    State { main_window }
}
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    use slint::android::android_activity::{MainEvent, PollEvent};
    slint::android::init_with_event_listener(app, |event| {
        match event {
            PollEvent::Main(MainEvent::SaveState { saver, .. }) => {
                STATE.with(|state| -> Option<()> {
                    let audiody_state = SerializedState::save(state.borrow().as_ref()?);
                    saver.store(&serde_json::to_vec(&audiody_state).ok()?);
                    Some(())
                });
            }
            PollEvent::Main(MainEvent::Resume { loader, .. }) => {
                STATE.with(|state| -> Option<()> {
                    let bytes: Vec<u8> = loader.load()?;
                    let audiody_state: SerializedState = serde_json::from_slice(&bytes).ok()?;
                    audiody_state.restore(state.borrow().as_ref()?);
                    Some(())
                });
            }
            _ => {}
        };
    })
    .unwrap();
    main();
}

pub struct State {
    pub main_window: AppWindow,
}

#[cfg(target_os = "android")]
thread_local! {
    static STATE : core::cell::RefCell<Option<State>> = Default::default();
}

#[cfg(target_os = "android")]
#[derive(serde::Serialize, serde::Deserialize)]
struct SerializedState {
    items: Vec<TodoItem>,
    sort: bool,
    hide_done: bool,
}

#[cfg(target_os = "android")]
impl SerializedState {
    fn restore(self, state: &State) {
        state.todo_model.set_vec(self.items);
        state.main_window.set_hide_done_items(self.hide_done);
        state.main_window.set_is_sort_by_name(self.sort);
        state.main_window.invoke_apply_sorting_and_filtering();
    }
    fn save(state: &State) -> Self {
        Self {
            items: state.todo_model.iter().collect(),
            sort: state.main_window.get_is_sort_by_name(),
            hide_done: state.main_window.get_hide_done_items(),
        }
    }
}
