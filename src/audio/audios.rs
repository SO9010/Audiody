use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use tokio::time::error::Elapsed;

#[derive(Clone)]
pub struct AudioService {
    stream_handle: Arc<Mutex<Option<rodio::OutputStreamHandle>>>,
    sink: Arc<Mutex<Option<Sink>>>,
    command_tx: mpsc::Sender<AudioCommand>,
    playback_distance: Arc<Mutex<u64>>,
}

// Commands for audio control
enum AudioCommand {
    Start(String),
    Play,
    Pause,
    Speed(f32),
    RelativeSeek(i64),
    Volume(f32),
}

impl Default for AudioService {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioService {
    pub fn new() -> Self {
        let stream_handle = Arc::new(Mutex::new(None));
        let sink = Arc::new(Mutex::new(None));
        let (command_tx, command_rx) = mpsc::channel();

        let sink_clone = sink.clone();
        let command_rx_clone = Arc::new(Mutex::new(command_rx));
        let playback_distance = Arc::new(Mutex::new(1));
        let playback_distance_clone = playback_distance.clone();

        thread::spawn(move || {
            let (_stream, handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&handle).unwrap();
            *sink_clone.lock().unwrap() = Some(sink);

            // Wait for commands
            loop {
                if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                    *playback_distance_clone.lock().unwrap() = sink.get_pos().as_secs();
                }
                match command_rx_clone.lock().unwrap().recv() {
                    Ok(AudioCommand::Start(path)) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            let file = BufReader::new(File::open(path).unwrap());
                            let source = Decoder::new(file).unwrap();
                            sink.clear();
                            sink.append(source);
                            sink.pause();
                        }
                    }
                    Ok(AudioCommand::Play) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            sink.play();
                        }
                    }
                    Ok(AudioCommand::Pause) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            sink.pause();
                        }
                    }
                    Ok(AudioCommand::Speed(speed)) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            sink.set_speed(speed);
                        }
                    }
                    Ok(AudioCommand::RelativeSeek(seconds)) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            let current_pos = sink.get_pos().as_secs();
                            let new_pos = (current_pos as i64 + seconds).max(0);
                            log::info!("Seeking to: {}", new_pos);
                            sink.try_seek(std::time::Duration::from_secs(new_pos as u64));
                        }
                    }
                    Ok(AudioCommand::Volume(volume)) => {
                        if let Some(sink) = sink_clone.lock().unwrap().as_ref() {
                            sink.set_volume(volume);
                        }
                    }
                    Err(_) => break, // Channel closed, exit thread
                }
            }
        });

        Self {
            stream_handle,
            sink,
            command_tx,
            playback_distance,
        }
    }

    pub fn start(&self, path: String) {
        // Send a signal to the audio thread to pause
        self.command_tx.send(AudioCommand::Start(path)).unwrap();
    }

    pub fn pause(&self) {
        // Send a signal to the audio thread to pause
        self.command_tx.send(AudioCommand::Pause).unwrap();
    }

    pub fn play(&self) {
        // Send a signal to the audio thread to play
        self.command_tx.send(AudioCommand::Play).unwrap();
    }

    pub fn set_speed(&self, speed: f32) {
        // Send a signal to the audio thread to set the speed
        self.command_tx.send(AudioCommand::Speed(speed)).unwrap();
    }

    pub fn seek_relative(&self, seconds: i64) {
        // Send a signal to the audio thread to set the speed
        self.command_tx
            .send(AudioCommand::RelativeSeek(seconds))
            .unwrap();
    }

    pub fn set_volume(&self, volume: f32) {
        // Send a signal to the audio thread to set the speed
        self.command_tx
            .send(AudioCommand::Volume(volume.min(100.0)))
            .unwrap();
    }

    // Need to implement!
    // pub fn get_chapter_len(&self, chapter_path: &str) -> u64 {}

    pub fn get_current_pos(&self) -> u64 {
        *self.playback_distance.lock().unwrap()
    }
}
