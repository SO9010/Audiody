use audio::audios::AudioService;
use slint::ComponentHandle;
use std::path;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use std::{path::PathBuf, vec};

pub mod api;
pub mod audio;
pub mod storage;

use api::{webapi::WebApiClient, webimage::url_to_buffer};
use storage::save::{download_audio, get_progress, save_progress};
use storage::saved::get_saved_book;
use storage::saved::get_saved_books;
use storage::setup::music_dir;
use tokio::runtime::{Handle, Runtime}; // 0.3.5

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

    let main_window: AppWindow = AppWindow::new().unwrap();
    let audio_state: AudioState<'_> = main_window.global::<AudioState>();
    let audio_service = AudioService::new();
    let webapi_client = WebApiClient::new();
    let previous_views = Arc::from(Mutex::new(vec![0]));

    // Set up the download folder
    music_dir();

    handle_ui_actions(
        &main_window,
        &audio_state,
        previous_views,
        &audio_service,
        &webapi_client,
    );

    State { main_window }
}

// UI Handling
fn handle_ui_actions(
    main_window: &AppWindow,
    audio_state: &AudioState<'_>,
    previous_views: Arc<Mutex<Vec<i32>>>,
    audio_service: &AudioService,
    webapi_client: &WebApiClient,
) {
    let mut inital_playback_distance: u64 = 0;
    // Get saved books:
    handle_saved_books(main_window);

    handle_previous_page_navigate(main_window, audio_state, Arc::clone(&previous_views));

    handle_page_navigate(audio_state, Arc::clone(&previous_views));

    handle_search(main_window, audio_state, webapi_client);

    handle_chapter_download_and_play(main_window, audio_state, audio_service);

    handle_chapter_download(main_window, audio_state);

    handle_book_view(main_window, audio_state, webapi_client);

    // Playback handles    
    handle_playing(main_window, audio_state, audio_service);

    handle_add_queue(main_window, audio_state, audio_service);

    handle_pause(main_window, audio_state, audio_service);

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
}

fn handle_saved_books(main_window: &AppWindow) {
    let saved_books = get_saved_books().unwrap();
    let main_window_weak = main_window.as_weak();

    thread::spawn(move || {
        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
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

            let books = slint::ModelRc::new(slint::VecModel::from(saved_books_converted));
            main_window
                .global::<AudioState>()
                .set_home_page_books(books)
        });
    });
}

fn handle_previous_page_navigate(
    main_window: &AppWindow,
    audio_state: &AudioState<'_>,
    previous_views: Arc<Mutex<Vec<i32>>>,
) {
    let main_window_weak = main_window.as_weak();
    audio_state.on_go_to_previous_page(move || {
        let main_window_weak = main_window_weak.clone();
        let previous_views = previous_views.clone();
        thread::spawn(move || {
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                // Sort it so the loading doesnt work
                if let Some(mut prev_view) = previous_views.lock().unwrap().pop() {
                    if prev_view == 100 {
                        prev_view = previous_views.lock().unwrap().pop().unwrap();
                    }
                    if prev_view == 0 {
                        main_window
                            .global::<AudioState>()
                            .set_page_name("Audiody".into());
                    }
                    main_window
                        .global::<AudioState>()
                        .set_current_view(prev_view)
                }
            });
        });
    });
}

fn handle_page_navigate(audio_state: &AudioState<'_>, previous_views: Arc<Mutex<Vec<i32>>>) {
    audio_state.on_add_previous_page(move |page| {
        previous_views.lock().unwrap().push(page);
    });
}

fn handle_search(
    main_window: &AppWindow,
    audio_state: &AudioState<'_>,
    webapi_client: &WebApiClient,
) {
    let main_window_weak = main_window.as_weak();
    let webapi_client_clone = webapi_client.clone();
    audio_state.on_on_search_clicked(move |query| {
        let webapi_client_clone = webapi_client_clone.clone();
        let main_window_weak = main_window_weak.clone();
        main_window_weak
            .upgrade()
            .unwrap()
            .global::<AudioState>()
            .set_current_view(100);
        thread::spawn(move || {
            let webapi_client_clone = webapi_client_clone.clone();

            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                let books = Runtime::new()
                    .unwrap()
                    .block_on(webapi_client_clone.search(query.to_string()))
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

                main_window
                    .global::<AudioState>()
                    .set_search_libi(libri_book_model);
                main_window
                    .global::<AudioState>()
                    .set_search_yt(yt_book_model);
                main_window.global::<AudioState>().set_current_view(1);
            });
        });
    });
}

fn handle_pause(main_window: &AppWindow, audio_state: &AudioState, audio_service: &AudioService) {
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
}

fn handle_add_queue(main_window: &AppWindow,
    audio_state: &AudioState,
    audio_service: &AudioService,
) {
    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_queue_next_track(move || {
        let audio_service_clone = audio_service_clone.clone();
        let main_window_weak = main_window_weak.clone();
        thread::spawn(move || {
            let audio_service_clone = audio_service_clone.clone();
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                main_window.global::<AudioState>().get_now_playing().book_url;
            });
    
            // audio_service_clone.queue();
        });
    });
}

fn handle_chapter_download_and_play(
    main_window: &AppWindow,
    audio_state: &AudioState,
    audio_service: &AudioService,
) {
    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_chapter_download_and_play(move |book, chapter, URL| {
        let audio_service_clone = audio_service_clone.clone();
        let main_window_weak = main_window_weak.clone();
        thread::spawn(move || {
            let audio_service_clone = audio_service_clone.clone();
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                match download_audio(
                    &book,
                    chapter,
                    &URL,
                    &main_window
                        .global::<AudioState>()
                        .get_book_view()
                        .book_url
                        .to_string(),
                ) {
                    Ok(path_buf) => {
                        if let Some(path_str) = path_buf.to_str() {
                            log::info!("Downloaded audio path: {}", path_str);
                            log::info!("Starting to play!!");
                            audio_service_clone.start(path_str.to_string());
                            audio_service_clone.play();
                            main_window.global::<AudioState>().set_paused(false);
                            // TODO: Optimise this so that it doesnt refresh the whole thing
                            main_window.global::<AudioState>().get_home_page_books();
                            let child = path_buf.file_name()
                            .unwrap()
                            .to_ascii_lowercase()
                            .into_string()
                            .unwrap()
                            .split_off(8); // Remove first 8 chars ("chapter_")
                        
                            let chapter_number = child[..child.len()-4].to_string(); // Remove last 4 chars (".mp3")
                            if let Ok(chapter_num) = chapter_number.parse::<i32>() {
                                save_progress(&main_window.global::<AudioState>().get_now_playing().title, Some(chapter_num), &main_window.global::<AudioState>().get_now_playing().book_url.as_str(), None);
                            } else {
                                log::error!("Failed to parse chapter number: {}", chapter_number);
                            }
                        } else {
                            log::info!("Error: Path contains invalid UTF-8");
                        }
                    }
                    Err(e) => {
                        log::info!("Error downloading audio: {}", e);
                    }
                }
            });
        });
    })
}
fn handle_chapter_download(main_window: &AppWindow, audio_state: &AudioState) {
    let main_window_weak = main_window.as_weak();
    audio_state.on_chapter_download(move |book, chapter, URL| {
        let main_window_weak = main_window_weak.clone();
        thread::spawn(move || {
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                match download_audio(
                    &book,
                    chapter,
                    &URL,
                    &main_window
                        .global::<AudioState>()
                        .get_book_view()
                        .book_url
                        .to_string(),
                ) {
                    Ok(path_buf) => {
                        if let Some(path_str) = path_buf.to_str() {
                            log::info!("Downloaded audio path: {}", path_str);
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
            });
        });
    })
}

fn handle_book_view(
    main_window: &AppWindow,
    audio_state: &AudioState<'_>,
    webapi_client: &WebApiClient,
) {
    let main_window_weak = main_window.as_weak();
    let webapi_client = webapi_client.clone();
    audio_state.on_on_book_view(move |book_url, book_title| {
        let main_window_weak = main_window_weak.clone();
        let webapi_client_clone = webapi_client.clone();
        main_window_weak
            .upgrade()
            .unwrap()
            .global::<AudioState>()
            .set_current_view(100);
        thread::spawn(move || {
            let webapi_client_clone = webapi_client_clone.clone();
            let main_window_weak = main_window_weak.clone();

            let mut book: api::types::Book = Default::default();
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                if let Ok(Some(book_thing)) = get_saved_book(book_title.to_string()) {
                    book = book_thing;

                    let mut book_online = Runtime::new()
                        .unwrap()
                        .block_on(webapi_client_clone.clone().get_book(book_url.to_string()))
                        .unwrap();

                    if book.chapter_urls.len() != book_online.chapter_urls.len() {
                        book_online.image_URL = book.image_URL;
                        book_online.saved = true;
                        book = book_online;
                    }
                } else {
                    book = Runtime::new()
                        .unwrap()
                        .block_on(webapi_client_clone.clone().get_book(book_url.to_string()))
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

                main_window.global::<AudioState>().set_book_view(book_item);
                main_window.global::<AudioState>().set_current_view(5);
            });
        });
    });
}

fn handle_playing(
    main_window: &AppWindow,
    audio_state: &AudioState<'_>,
    audio_service: &AudioService,
) {
    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_playing_action(move || {
        let main_window_weak = main_window_weak.clone();
        let audio_service_clone = audio_service_clone.clone();
        thread::spawn(move || {
            let audio_service_clone = audio_service_clone.clone();
            let main_window_weak = main_window_weak.clone();

            // Make this far more efficient
            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                let audio_path = music_dir().unwrap().as_path().join(main_window.global::<AudioState>().get_now_playing().title).join(format!{"chapter_{}.mp3", get_progress(&main_window.global::<AudioState>().get_now_playing().title).unwrap().current_chapter.unwrap()});

                let current_pos = audio_service_clone.get_current_pos() as f32;
                let chapter_len = audio_service_clone.get_chapter_len(audio_path.clone().to_str().unwrap()).as_secs_f32();

                main_window.global::<AudioState>().set_timing(
                    current_pos / chapter_len
                );
                save_progress(
                    &main_window.global::<AudioState>().get_now_playing().title,
                    get_progress(&main_window.global::<AudioState>().get_now_playing().title).unwrap().current_chapter,
                    &main_window.global::<AudioState>().get_now_playing().book_url.as_str(),
                    Some((current_pos / chapter_len) as f64)
                );
            });
        });
    });
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
