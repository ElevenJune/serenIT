use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread;

pub struct Sound {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    source: String,
    volume: f32,
    loaded: bool,
}

impl Sound {
    pub fn new(source: &String, volume: f32) -> Sound {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => (stream, handle),
            Err(_) => panic!("Failed to create stream"),
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(_) => panic!("Failed to create sink"),
        };
        Sound {
            stream,
            stream_handle,
            sink,
            source: source.clone(),
            volume,
            loaded: false,
        }
    }

    pub fn load(&mut self) {
        if self.loaded {
            return;
        }
        let file: BufReader<File> = BufReader::new(File::open(&self.source).unwrap());
        let buffer = Decoder::new(file).unwrap();
        self.sink.append(buffer);
        println!("Playing sound: {} with volume {}", self.source, self.volume);
        self.loaded = true;
    }

    pub fn play(&mut self) {
        self.load();
        self.set_volume(self.volume);
        self.sink.play();
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
}


fn main() {

    let mut sounds: Vec<Sound> = vec![];
    let params: Vec<(String, f32)> = vec![
        ("./sounds/nature/waves.mp3".to_string(), 1.),
        ("./sounds/things/wind-chimes.mp3".to_string(), 1.),
        ("./sounds/binaural/binaural-alpha.wav".to_string(), 0.01),
    ];

    // Create Sound instances with separate sinks for each sound
    for (source, volume) in params.iter() {
        sounds.push(Sound::new(&source, *volume));
        sounds.last_mut().unwrap().load();
    }

    // Play all sounds in the main thread
    for sound in sounds.iter_mut() {
        sound.play();
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
}
