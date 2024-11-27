use std::rc::Rc;
use slint::ComponentHandle;
pub mod api;
use api::{librivox::{self, LibriVoxClient}, types::{Book, SearchQuery}, webimage::url_to_buffer};


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

#[tokio::main]
async fn init() -> State {
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let main_window = AppWindow::new().unwrap();
    let librivox_client = Rc::new(LibriVoxClient::new());

    let audio_state = main_window.global::<AudioState>();

    // At somepoint we want to implement multi threading so that the requests dont get int the way of the UI
    // We also need to implement caching!
    let books = librivox_client.search("Marxism".to_string()).unwrap();

    let book_items: Vec<BookItem> = books.into_iter().map(|book| BookItem {title:book.title.into(),author:book.author.into(),image_url:book.image_URL.clone().into(),book_url:book.url.into(),saved:book.saved,image:url_to_buffer(book.image_URL).unwrap(), chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])) }).collect();

    let book_model = Rc::new(slint::VecModel::<BookItem>::from(book_items));
    main_window.global::<AudioState>().set_search_model(book_model.clone().into());
    
    let librivox_client_clone = librivox_client.clone();
    let main_window_weak = main_window.as_weak();

    audio_state.on_on_search_clicked(move |query| {
        // We also need to implement caching!
        let libri = librivox_client_clone.clone();
        let main_window_weak = main_window_weak.clone();
        let books = libri.search(query.to_string()).unwrap();

        let book_items: Vec<BookItem> = books.into_iter().map(|book| BookItem {title:book.title.into(),author:book.author.into(),image_url:book.image_URL.clone().into(),book_url:book.url.into(),saved:book.saved,image:url_to_buffer(book.image_URL).unwrap(), chapter_urls: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_durations: slint::ModelRc::new(slint::VecModel::from(vec![])), chapter_reader: slint::ModelRc::new(slint::VecModel::from(vec![])) }).collect();

        let book_model = Rc::new(slint::VecModel::<BookItem>::from(book_items));
        if let Some(main_window) = main_window_weak.upgrade() {
            main_window.global::<AudioState>().set_search_model(book_model.clone().into());
        }
    });

    let librivox_client_clone = librivox_client.clone();
    let main_window_weak = main_window.as_weak();

    audio_state.on_on_book_view(move |bookURL| {
        // We also need to implement caching!
        let libri = librivox_client_clone.clone();
        let main_window_weak = main_window_weak.clone();
    
        let book = libri.get_book(bookURL.to_string()).unwrap();

        let book_item: BookItem = BookItem {
            title: book.title.into(),
            author: book.author.into(),
            image_url: book.image_URL.clone().into(),
            book_url: book.url.into(),
            saved: book.saved,
            image: url_to_buffer(book.image_URL).unwrap(),
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
