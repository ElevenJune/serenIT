use rodio::source::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;

pub struct SinkHandle {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink
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
            sink
        }
    }

    pub fn is_playing(&self) -> bool {
        self.sink.len()!=0
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn play_raw<'a>(&mut self, data: std::io::Cursor<&'static [u8]>) {
        self.clear_if_playing();
        let source = Decoder::new(Cursor::new(data.into_inner())).unwrap();
        self.sink.append(source.repeat_infinite());
    }


    pub fn set_source(&mut self, source: &str) {
        self.clear_if_playing();
        self.add_to_queue(source);
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn play(&mut self) {
        self.sink.play();
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.sink.clear();
    }

    fn add_to_queue(&mut self, source: &str) {
        let file: BufReader<File> = BufReader::new(File::open(source).unwrap());
        let buffer = Decoder::new(file).unwrap();
        self.sink.append(buffer.repeat_infinite());
    }

    fn clear_if_playing(&mut self) {
        if self.is_playing() {
            self.stop();
        }
    }
}
