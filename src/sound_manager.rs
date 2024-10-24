use cli_log::*;
use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use std::thread;
use std::sync::mpsc;

use crate::sound::Sound;

const MAX_SOUNDS: usize = 4;

pub struct SoundManager {
    sounds: Vec<Sound>,
    available_sounds: Vec<(String,String)>,
    //thread: thread::JoinHandle<()>
}

pub enum SoundManagerError {
    NoAvailableSound,
    AlreadyPlaying,
    OtherError,
}

impl SoundManager {
    pub fn new() -> Self {
        let mut sounds = vec![];
        for _i in 0..MAX_SOUNDS {
            sounds.push(Sound::new());
        }
        let mut sm = SoundManager { sounds, available_sounds:vec![] };
        sm.load_available_sounds();
        sm
    }

    pub fn get_sound_list(&self) -> &Vec<(String,String)>{
        &self.available_sounds
    }

    pub fn is_source_playing(&self,source:&String) -> bool{
        let res = self.sounds
        .iter()
        .find(|s| s.get_source()==source && s.is_playing());
        //info!("{} is playing : {}",source, res.is_some());
        res.is_some()
    }

    pub fn add_sound(&mut self, source: &String) -> Result<(), SoundManagerError> {
        self.update_all();
        if self.is_source_playing(source){
            warn!("{} is already playing",source);
            return Err(SoundManagerError::AlreadyPlaying);
        }
        let res = match self.find_available() {
            Some(i) => {
                info!("Playing {}", i);
                self.sounds[i].set_source(source);
                self.sounds[i].play();
                Ok(())
            }
            None => {
                self.overwrite_last(source);
                warn!("{} overwrote last sound",source);
                Err(SoundManagerError::NoAvailableSound)
            }
        };
        self.update_all();
        res
    }

    pub fn remove_sound(&mut self, source: &String){
        let res = self.sounds
        .iter_mut()
        .find(|s| s.get_source()==source && s.is_playing());
        match res{
            Some(s)=>{s.stop();}
            None=>{}
        };
        self.update_all();
    }

    pub fn adjust_volume(&mut self, source: &String, volume_offset:f32){
        let res = self.sounds
        .iter_mut()
        .find(|s| s.get_source()==source && s.is_playing());
        match res{
            Some(s)=>{s.adjust_volume(volume_offset);}
            None=>{}
        };
    }

    pub fn get_volume(&self) -> f32{
        self.sounds[0].volume()
    }

    pub fn sounds(&self) -> &Vec<Sound>{
        &self.sounds
    }

    fn overwrite_last(&mut self, source: &String) {
        match self.sounds.first_mut() {
            Some(s) => {
                s.stop();
                s.set_source(source);
                s.play();
            }
            None => {}
        }
    }

    fn find_available(&self) -> Option<usize> {
        self.sounds
            .iter()
            .enumerate()
            .find(|(_, s)| !s.is_playing())
            .map(|(i, _)| i)
    }

    pub fn update_all(&mut self) {
        self.sounds.iter_mut().for_each(|s: &mut Sound| s.update());
    }

    fn load_available_sounds(&mut self){
        self.available_sounds.clear();
        let params: Vec<String> = vec![
            "./sounds/nature/waves.mp3".to_string(),
            "./sounds/things/wind-chimes.mp3".to_string(),
            "./sounds/binaural/binaural-alpha.wav".to_string(),
        ];
        params.iter().for_each(|path|{
            let filename = path.split('/').last().unwrap();
            self.available_sounds.push((path.clone(),filename.to_string()));
        });
    }
}
