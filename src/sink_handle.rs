use rodio::source::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
pub struct SinkHandle {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    source: String,
    loaded: bool,
    is_playing: bool,
}

impl SinkHandle {
    pub fn new() -> SinkHandle {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((stream, handle)) => (stream, handle),
            Err(_) => panic!("Failed to create stream"),
        };
        let sink = match Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(_) => panic!("Failed to create sink"),
        };
        SinkHandle {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            source: "".to_string(),
            loaded: false,
            is_playing: false,
        }
    }

    pub fn set_source(&mut self, source: &String) {
        self.clear_if_playing();
        self.source = source.clone();
        self.add_to_queue(&self.source.clone());
    }

    pub fn get_source(&self) -> &String {
        &self.source
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    fn load(&mut self) {
        if self.loaded {
            return;
        }
        
        self.loaded = true;
    }

    pub fn add_to_queue(&mut self, source: &String) {
        let file: BufReader<File> = BufReader::new(File::open(source).unwrap());
        let buffer = Decoder::new(file).unwrap();
        self.sink.append(buffer.repeat_infinite());
    }

    pub fn update(&mut self) {
        let queue_size = self.sink.len();
        self.is_playing = queue_size != 0;
    }

    pub fn play(&mut self) {
        self.sink.play();
        self.update();
    }

    pub fn is_playing(&self) -> bool {
        self.sink.len()!=0
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.sink.clear();
        self.update();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    fn clear_if_playing(&mut self) {
        if self.is_playing() {
            self.stop();
        }
    }
}
