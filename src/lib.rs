use std::sync::mpsc;
use std::{rc::Rc, sync::Arc, thread, vec};
use audio::audios::AudioService;
use slint::{ComponentHandle, SharedString, VecModel};

pub mod api;
pub mod audio;

use api::{librivox::{self, LibriVoxClient}, types::{Book, SearchQuery}, webapi::WebApiClient, webimage::url_to_buffer, yt::{self, YouTubeClient}};
use tokio::{runtime::Runtime, sync::{broadcast, Mutex}}; // 0.3.5
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};

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
    tokio::task::block_in_place(||main_window.run().unwrap());
}

fn init() -> State {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let main_window = AppWindow::new().unwrap();
    let audio_state = main_window.global::<AudioState>();
    let audio_service = AudioService::new();
    let webapi_client = WebApiClient::new();
    // Implement populating the books one by one do then it looks more dynamic and doesnt just wait ages
    // At somepoint we want to implement multi threading so that the requests dont get int the way of the UI
    let previous_views = Rc::new(std::cell::RefCell::new(vec![0]));
    
    let main_window_weak = main_window.as_weak();
    let previous_views_clone = previous_views.clone();
    audio_state.on_go_to_previous_page(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            if let Some(prev_view) = previous_views_clone.borrow_mut().pop() {
                if (prev_view == 0) {
                    main_window.global::<AudioState>().set_page_name("Audiody".into());
                }
                main_window.global::<AudioState>().set_current_view(prev_view.into())
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
        let books = Runtime::new().unwrap().block_on(webapi_client.clone().search(query.to_string())).unwrap();
        let mut libri_book_items: Vec<BookItem>; 
        libri_book_items = books[0].clone().into_iter().map(|book| BookItem {title:book.title.into(),author:book.author.into(),description:book.description.clone().into(),book_url:book.url.into(),saved:book.saved,image:Runtime::new().unwrap().block_on(url_to_buffer(book.image_URL)).unwrap(), chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])) }).collect();
        let libri_book_model = Rc::new(slint::VecModel::<BookItem>::from(libri_book_items));

        let yt_book_items: Vec<BookItem> = books[1].clone().into_iter().map(|book| BookItem {title:book.title.into(),author:book.author.into(),description:book.description.clone().into(),book_url:book.url.into(),saved:book.saved,image:Runtime::new().unwrap().block_on(url_to_buffer(book.image_URL)).unwrap(), chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])) }).collect();
        let yt_book_model = Rc::new(slint::VecModel::<BookItem>::from(yt_book_items));

        if let Some(main_window) = main_window_weak.upgrade() {
            main_window.global::<AudioState>().set_search_libi(libri_book_model.clone().into());
            main_window.global::<AudioState>().set_search_yt(yt_book_model.clone().into());
        }
    });

    let main_window_weak = main_window.as_weak();
    let audio_service_clone = audio_service.clone();
    audio_state.on_toggle_pause(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            if !main_window.global::<AudioState>().get_paused() {
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
    let webapi_client = WebApiClient::new();
    audio_state.on_on_book_view(move |bookURL| {
        // We also need to implement caching!
        let main_window_weak = main_window_weak.clone();
    
        let book = Runtime::new().unwrap().block_on(webapi_client.clone().get_book(bookURL.to_string())).unwrap();
        
        let book_item: BookItem = BookItem {
            title: book.title.into(),
            author: book.author.into(),
            description: book.description.clone().into(),
            book_url: book.url.into(),
            saved: book.saved,
            image: Runtime::new().unwrap().block_on(url_to_buffer(book.image_URL)).unwrap(),
            // Find a better way of doing this
            chapter_urls: slint::ModelRc::new(slint::VecModel::from(book.chapter_urls.into_iter().map(|url| url.into()).collect::<Vec<slint::SharedString>>())),
            chapter_durations: slint::ModelRc::new(slint::VecModel::from(book.chapter_durations.into_iter().map(|dur| dur.into()).collect::<Vec<slint::SharedString>>())),
            chapter_reader: slint::ModelRc::new(slint::VecModel::from(book.chapter_reader.into_iter().map(|reader| reader.into()).collect::<Vec<slint::SharedString>>()))
        };

        if let Some(main_window) = main_window_weak.upgrade() {
            main_window.global::<AudioState>().set_book_view(book_item);
        }
    });

    State { main_window }
}

#[cfg(target_os = "android")]#[no_mangle]
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
