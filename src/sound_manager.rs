use crate::sink_handle::SinkHandle;
use crate::sound::Sound;
use cli_log::*;
use homedir::my_home;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use thiserror::Error;
use std::path::Path;

const MAX_SOUNDS: usize = 8;

pub struct SoundManager {
    available_sounds: Vec<Sound>,
    sinks: Vec<SinkHandle>,
    playing_sounds: HashMap<String, usize>,
    config_path: String,
    categories: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SoundData {
    pub source: String,
    pub volume: f32,
}

pub enum SoundManagerError {
    NoAvailableSound,
    AlreadyPlaying,
    AlreadyStopped,
    SoundDoesNotExists,
    OtherError,
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl SoundManager {
    pub fn new() -> Self {
        let mut sinks = vec![];
        for _i in 0..MAX_SOUNDS {
            sinks.push(SinkHandle::new());
        }
        let mut sm = SoundManager {
            sinks,
            available_sounds: vec![],
            playing_sounds: HashMap::new(),
            config_path: "".to_string(),
            categories: vec![],
        };
        sm.load_available_sounds();
        sm.load_presets().unwrap_or_else(|err| {
            warn!("No presets found, {}. Loading default demo",err);
            sm.demo();
        });
        sm
    }

    //===== Getters
    pub fn get_sound_list(&self) -> &Vec<Sound> {
        &self.available_sounds
    }

    pub fn playing_sounds(&self) -> &HashMap<String, usize> {
        &self.playing_sounds
    }

    pub fn categories(&self) -> &Vec<String> {
        &self.categories
    }

    pub fn is_sound_playing(&self, path: &str) -> bool {
        if !self.playing_sounds.contains_key(path) {
            return false;
        }
        let sink_index = self.playing_sounds.get(path).unwrap();
        self.sinks[*sink_index].is_playing()
    }

    pub fn get_sound_path_by_index(&self, index: usize) -> &str {
        self.available_sounds[index].path()
    }

    pub fn get_sound_path_by_index_and_category(&self, index: usize, category_index : Option<usize>) -> &str {
        let cat_index = self.available_sounds
        .iter()
        .enumerate()
        .find(|(_,s)| {
            match category_index {
                Some(i) => s.category() == self.categories[i],
                None => true
            }
        });

        match cat_index {
            Some((i,_s)) => self.available_sounds[i+index].path(),
            None => ""
        }
    }

    pub fn get_sound_by_path(&self, path: &str) -> Option<&Sound> {
        self.available_sounds.iter().find(|s| s.path() == path)
    }

    //===== Actions
    pub fn toggle_sound(&mut self, path: &str) -> Result<(), SoundManagerError> {
        let res = if self.is_sound_playing(path) {
            self.remove_sound(path)
        } else {
            self.add_sound(path)
        };
        res
    }

    fn add_sound(&mut self, path: &str) -> Result<(), SoundManagerError> {
        let sound = self
            .get_sound_by_path(path)
            .ok_or(SoundManagerError::SoundDoesNotExists)?;

        let path = &path.to_string();
        let volume = sound.volume();

        // Find an available sink or overwrite the last one
        let sink_index = match self.find_available() {
            Some(i) => i,
            None => {
                self.overwrite_last(path, volume);
                return Err(SoundManagerError::NoAvailableSound);
            }
        };

        // Set the source and volume of the found sink
        self.set_sink_source(sink_index, path, volume);
        Ok(())
    }

    fn remove_sound(&mut self, path: &str) -> Result<(), SoundManagerError> {
        if let Some(_) = self.get_sound_by_path(path) {
            let path = &path.to_string();
            match self.playing_sounds.get(path) {
                Some(i) => {
                    info!("Sound {} from sink {} stopped", path, i);
                    self.sinks[*i].stop();
                    self.playing_sounds.remove(path);
                    Ok(())
                }
                None => Err(SoundManagerError::AlreadyStopped),
            }
        } else {
            Err(SoundManagerError::SoundDoesNotExists)
        }
    }

    pub fn adjust_sound_volume(&mut self, path: &str, volume_offset: f32) {
        self.adjust_volume(path, volume_offset, false);
    }

    pub fn adjust_master_volume(&mut self, volume_offset: f32){
        let sounds:Vec<String> = self.playing_sounds.keys().map(|path| path.clone()).collect();
        sounds.iter().for_each(|path| {
            self.adjust_volume(path, volume_offset, true);
        });
    }

    fn overwrite_last(&mut self, source: &String, volume: f32) {
        let mut path = "".to_string();
        let mut sink_index = MAX_SOUNDS;
        self.playing_sounds.keys().for_each(|p| {
            let index = self.playing_sounds.get(p).unwrap().clone();
            if index == self.sinks.len() - 1 {
                path = p.clone();
                sink_index = index;
            }
        });
        self.playing_sounds.remove(&path);
        self.set_sink_source(sink_index, source, volume);
    }

    pub fn save(&mut self) -> Result<(), FileError> {
        if self.config_path.is_empty() {
            return Err(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "No config path found")));
        }
        self.save_to(self.config_path.clone())
    }

    //===== Misc
    fn find_available(&self) -> Option<usize> {
        self.sinks
            .iter()
            .enumerate()
            .find(|(_, s)| !s.is_playing())
            .map(|(i, _)| i)
    }

    fn demo(&mut self) {
        let params = vec![
            ("waves.mp3", -0.3),
            ("birds.mp3", -0.4),
            ("wind-chimes.mp3", -0.6),
            ("binaural-alpha.wav", -0.15),
        ];
        info!("Demo: {:?}", params);
        params.iter().for_each(|(name, volume)| {
            let _ = self.toggle_sound(name);
            self.adjust_volume(name, *volume, false);
        });
    }

    fn adjust_volume(&mut self, path: &str, volume_offset: f32, master:bool) {
        // Find the sound to adjust
        if let Some(sound) = self.available_sounds.iter_mut().find(|s| s.path() == path) {
            // Calculate the new volume
            let mut new_volume = sound.volume() + volume_offset;
            new_volume = new_volume.clamp(if master {0.02} else {0.0}, 1.0);

            // Update the sound's volume
            sound.set_volume(new_volume);

            // Find the corresponding sink and update its volume
            if let Some(i) = self.playing_sounds.get(sound.path()) {
                self.sinks[*i].set_volume(new_volume);
            }
        }
    }

    fn set_sink_source(&mut self, sink_index: usize, path: &String, volume: f32) {
        info!("Playing sound {} to sink {}", path, sink_index);
        self.playing_sounds.insert(path.clone(), sink_index);

        let sink = &mut self.sinks[sink_index];
        sink.set_volume(volume);
        sink.set_source(path);
        sink.play();
    }

    fn load_presets(&mut self) -> Result<(), FileError> {
        let home_dir = my_home().map_err(|e| FileError::IoError(std::io::Error::new(std::io::ErrorKind::Unsupported, e)))?;
        let path = home_dir.ok_or_else(|| FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")))?
        .to_str()
        .unwrap()
        .to_string() + "/.config/moodist/sounds.json";
    
        let res = self.read_from_file(&path);
        if res.is_ok() {
            self.config_path = path;
        }
        res
    }

    fn save_to(&self, path: String) -> Result<(), FileError> {
        // Create/open the file
        let file_path = Path::new(&path);
        let parent_dir = file_path.parent().ok_or(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "No parent directory found")))?;
        std::fs::create_dir_all(parent_dir)?;
        let mut f = File::create(path)?;

        // Serialize the struct
        let mut config:Vec<SoundData> = vec![];
        self.playing_sounds
            .keys()
            .for_each(|path| {
                if let Some(sound) = self.get_sound_by_path(path){
                config.push(SoundData {
                    source: path.clone(),
                    volume: sound.volume(),
                });
            }
            });
        let serialized = serde_json::to_string(&config)?;

        info!("Saving to file: {}", serialized);

        // Write to file
        f.write_all(serialized.as_bytes())?;

        Ok(())
    }

    fn read_from_file(&mut self, path: &str) -> Result<(), FileError> {
        let mut file = File::open(path)?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        let config: Vec<SoundData> = serde_json::from_str(&buff)?;
        for (i, s) in config.iter().enumerate() {
            if s.source.len() > 0 {
                info!("Loading from file: {}, with volume {}", s.source, s.volume);
                self.available_sounds.iter_mut()
                .find(|sound| sound.path() == s.source)
                .map(|sound| sound.set_volume(s.volume));

                self.set_sink_source(i, &s.source, s.volume);
            }
        }
        Ok(())
    }

    fn load_available_sounds(&mut self) {
        self.available_sounds.clear();
        let params: Vec<&str> = vec![
            // sounds/
            "./sounds/animals/birds.mp3",
            "./sounds/animals/cat-purring.mp3",
            "./sounds/animals/crickets.mp3",
            "./sounds/animals/crows.mp3",
            "./sounds/animals/dog-barking.mp3",
            "./sounds/animals/frog.mp3",
            "./sounds/animals/horse-galopp.mp3",
            "./sounds/animals/owl.mp3",
            "./sounds/animals/seagulls.mp3",
            "./sounds/animals/whale.mp3",
            "./sounds/animals/wolf.mp3",
            "./sounds/binaural/binaural-alpha.wav",
            "./sounds/binaural/binaural-beta.wav",
            "./sounds/binaural/binaural-delta.wav",
            "./sounds/binaural/binaural-gamma.wav",
            "./sounds/binaural/binaural-theta.wav",
            "./sounds/nature/campfire.mp3",
            "./sounds/nature/droplets.mp3",
            "./sounds/nature/jungle.mp3",
            "./sounds/nature/river.mp3",
            "./sounds/nature/walk-in-snow.mp3",
            "./sounds/nature/walk-on-leaves.mp3",
            "./sounds/nature/waterfall.mp3",
            "./sounds/nature/waves.mp3",
            "./sounds/nature/wind.mp3",
            "./sounds/nature/wind-in-trees.mp3",
            "./sounds/noise/brown-noise.wav",
            "./sounds/noise/pink-noise.wav",
            "./sounds/noise/white-noise.wav",
            "./sounds/places/airport.mp3",
            "./sounds/places/cafe.mp3",
            "./sounds/places/carousel.mp3",
            "./sounds/places/church.mp3",
            "./sounds/places/construction-site.mp3",
            "./sounds/places/crowded-bar.mp3",
            "./sounds/places/laboratory.mp3",
            "./sounds/places/laundry-room.mp3",
            "./sounds/places/night-village.mp3",
            "./sounds/places/office.mp3",
            "./sounds/places/subway-station.mp3",
            "./sounds/places/supermarket.mp3",
            "./sounds/places/temple.mp3",
            "./sounds/places/underwater.mp3",
            "./sounds/rain/heavy-rain.mp3",
            "./sounds/rain/light-rain.mp3",
            "./sounds/rain/rain-on-leaves.mp3",
            "./sounds/rain/rain-on-tent.mp3",
            "./sounds/rain/rain-on-umbrella.mp3",
            "./sounds/rain/rain-on-window.mp3",
            "./sounds/rain/thunder.mp3",
            "./sounds/things/boiling-water.mp3",
            "./sounds/things/bubbles.mp3",
            "./sounds/things/ceiling-fan.mp3",
            "./sounds/things/clock.mp3",
            "./sounds/things/dryer.mp3",
            "./sounds/things/keyboard.mp3",
            "./sounds/things/morse-code.mp3",
            "./sounds/things/paper.mp3",
            "./sounds/things/slide-projector.mp3",
            "./sounds/things/singing-bowl.mp3",
            "./sounds/things/tuning-radio.mp3",
            "./sounds/things/typewriter.mp3",
            "./sounds/things/washing-machine.mp3",
            "./sounds/things/wind-chimes.mp3",
            "./sounds/transport/airplane.mp3",
            "./sounds/transport/inside-a-train.mp3",
            "./sounds/transport/rowing-boat.mp3",
            "./sounds/transport/sailboat.mp3",
            "./sounds/transport/submarine.mp3",
            "./sounds/transport/train.mp3",
            "./sounds/urban/ambulance-siren.mp3",
            "./sounds/urban/busy-street.mp3",
            "./sounds/urban/crowd.mp3",
            "./sounds/urban/fireworks.mp3",
            "./sounds/urban/highway.mp3",
            "./sounds/urban/road.mp3",
            "./sounds/urban/traffic.mp3",
            "./sounds/alarm.mp3",
        ];
        params.iter().for_each(|path| {
            let folders = path.split("/").collect::<Vec<&str>>();
            let filename = folders[folders.len() - 1];
            let category = folders[folders.len() - 2];
            let volume = if filename.contains("binaural") || filename.contains("noise") {
                0.2
            } else {
                0.5
            };
            self.available_sounds
                .push(Sound::new(filename, path, category, volume));
            if !self.categories.contains(&category.to_string()) {
                self.categories.push(category.to_string());
            }
        });
    }
}
