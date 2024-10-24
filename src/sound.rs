use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use cli_log::*;

pub struct Sound {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    source: String,
    volume: f32,
    loaded: bool,
    is_playing: bool,
    loop_sound:bool,
}

impl Sound {
    pub fn new() -> Sound {
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
            source: "".to_string(),
            volume:0.5,
            loaded: false,
            is_playing:false,
            loop_sound:true
        }
    }

    pub fn set_source(&mut self, source: &String){
        self.clear_if_playing();
        self.source=source.clone();
        self.loaded=false;
        self.load();
    }

    pub fn get_source(&self) -> &String{
        &self.source
    }

    pub fn volume(&self) -> f32{
        self.volume
    }

    pub fn load(&mut self) {
        if self.loaded {
            return;
        }
        self.add_to_queue(&self.source.clone());
        self.set_volume(self.volume);
        info!("Playing sound: {} with volume {}", self.source, self.volume);
        self.loaded = true;
    }

    pub fn add_to_queue(&mut self, source :&String){
        let file: BufReader<File> = BufReader::new(File::open(source).unwrap());
        let buffer = Decoder::new(file).unwrap();
        self.sink.append(buffer.repeat_infinite());
    }

    pub fn update(&mut self){
        let queue_size= self.sink.len();
        self.is_playing=queue_size!=0;
        /*if queue_size==1 && self.loop_sound && self.is_playing{
            info!("Adding source to queue");
            self.add_to_queue(&self.source.clone());
        }*/
    }

    pub fn adjust_volume(&mut self, offset:f32){
        let mut new_volume = self.volume+offset;
        if new_volume>2. {new_volume=2.;}
        if new_volume<0. {new_volume=0.;}
        self.set_volume(new_volume);
    }

    pub fn play(&mut self) {
        self.load();
        self.sink.play();
        self.update();
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.sink.clear();
        self.update();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
        self.volume=self.sink.volume();
    }

    fn clear_if_playing(&mut self){
        if self.is_playing(){
            self.stop();
        }
    }
}