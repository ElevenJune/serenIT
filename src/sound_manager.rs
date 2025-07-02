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
use std::include_bytes;
use std::io::Cursor;

const MAX_SOUNDS: usize = 8;

pub struct SoundManager {
    available_sounds: Vec<Sound>,
    categories: Vec<String>,
    sinks: Vec<SinkHandle>,
    playing_sounds: HashMap<String, usize>,
    scenes: Vec<SceneData>,
    current_scene_index: usize,
    config_path: String,
}

/// Represents the data of a scene
/// A scene is a collection of ambiant sounds with their volume
#[derive(Serialize, Deserialize, Clone)]
pub struct SceneData {
    pub name: String,
    sounds: Vec<SoundData>
}

/// Represents a sound data, its source path and volume
#[derive(Serialize, Deserialize, Clone)]
pub struct SoundData {
    pub source: String,
    pub volume: f32,
}

/// Represents the possible errors of the SoundManager
pub enum SoundManagerError {
    NoAvailableSound,
    AlreadyPlaying,
    AlreadyStopped,
    SoundDoesNotExists,
    OtherError,
}

/// Represents the possible errors while serializing/deserializing a file
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
            available_sounds: vec![],
            categories: vec![],
            sinks,
            playing_sounds: HashMap::new(),
            scenes: vec![],
            config_path: "".to_string(),
            current_scene_index: 0,
        };
        sm.load_sound_collection();
        sm.load_scene_collection().unwrap_or_else(|err| {
            warn!("No presets found, {}. Loading default demo",err);
            sm.create_empty_scene();
            sm.current_scene_index = 0;
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

    pub fn is_sound_paused(&self, path: &str) -> bool {
        if !self.playing_sounds.contains_key(path) {
            return false;
        }
        let sink_index = self.playing_sounds.get(path).unwrap();
        self.sinks[*sink_index].is_paused()
    }

    pub fn get_sound_path_by_index(&self, index: usize) -> &str {
        self.available_sounds[index].path()
    }

    pub fn get_scene_collection(&self) -> &Vec<SceneData> {
        &self.scenes
    }

    pub fn get_current_scene_index(&self) -> usize {
        self.current_scene_index
    }

    pub fn get_sound_path_by_index_and_category(&self, index: usize, category_index : Option<usize>) -> Option<&str> {
        //Find first element that matches the category
        let cat_index = self.available_sounds
        .iter()
        .enumerate()
        .find(|(_,s)| {
            match category_index {
                Some(i) => s.category() == self.categories[i],
                None => true
            }
        });
        //Get corresponding element of the category
        cat_index.and_then(|(i,_)| {
            if i+index >= self.available_sounds.len() {
                return None;
            }
            return Some(self.available_sounds[i+index].path());})
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

    pub fn toggle_pause_play(&mut self) {
        if self.sinks.iter().all(|sink| sink.is_paused()) {
            self.play_all();
        } else {
            self.pause_all();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.sinks.iter().all(|sink| sink.is_paused())
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

    pub fn save(&mut self) -> Result<(), FileError> {
        if self.config_path.is_empty() {
            return Err(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "No config path found")));
        }
        self.write_scene_collection_to(self.config_path.clone())
    }

    pub fn play_scene(&mut self, scene_index: usize) -> Result<(), FileError> {
        if scene_index >= self.scenes.len() {
            return Err(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Scene not found")));
        }
        
        // Clear current playing sounds
        self.playing_sounds.clear();
        self.sinks.iter_mut().for_each(|sink| sink.stop());
        self.current_scene_index = scene_index;

        let sounds = self.scenes[scene_index].sounds.clone();
        for (_i, sound_data) in sounds.iter().enumerate() {
            if !sound_data.source.is_empty() {
                info!("Playing sound {} with volume {}", sound_data.source, sound_data.volume);
                let _ = self.add_sound(&sound_data.source);
                let sound = self.get_sound_by_path(&sound_data.source).unwrap();
                let current_volume = sound.volume();
                self.adjust_sound_volume(&sound_data.source, sound_data.volume-current_volume);
            }
        }
        info!("Scene {} played", scene_index);
        Ok(())
    }

    pub fn create_empty_scene(&mut self) {
        let mut scene_index = 0;
        let mut new_scene = SceneData {
            name: format!("Scene_{}", scene_index),
            sounds: vec![],
        };
        let mut name_found = true;
        while name_found {
            name_found = false;
            for scene in &self.scenes {
                if scene.name == new_scene.name {
                    name_found = true;
                    new_scene.name = format!("Scene_{}", scene_index + 1);
                    break;
                }
            }
            scene_index += 1;
        }
        self.scenes.push(new_scene);
    }

    pub fn delete_scene(&mut self, scene_index:usize) {
        let current_len = self.scenes.len();
        if current_len <= 1 {return;}
        self.scenes.remove(scene_index);
        if self.current_scene_index == scene_index {
            let new_index = if scene_index < current_len-1 { scene_index } else { current_len-1 };
            let _ = self.play_scene(new_index);
            self.pause_all();
        } else {
            self.current_scene_index = if self.current_scene_index >= scene_index {self.current_scene_index-1} else {self.current_scene_index};
        }

    }

    //===== Misc
    fn find_available(&self) -> Option<usize> {
        self.sinks
            .iter()
            .enumerate()
            .find(|(_, s)| !s.is_playing())
            .map(|(i, _)| i)
    }

    fn pause_all(&mut self) {
        self.sinks.iter_mut().for_each(|sink| {
            sink.pause();
        });
    }

    fn play_all(&mut self) {
        self.sinks.iter_mut().for_each(|sink| {
            sink.play();
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
        match SOUNDS.iter().find(|(p,_)| p == path) {
            Some((_,data)) => {
                sink.set_source(data.clone(),path);
                sink.play();
            }
            None => {
                error!("Sound {} not found", path);
            }
        }
    }

    pub fn save_current_scene(&mut self){
        let scene_index = self.current_scene_index;
        if scene_index < self.scenes.len() {
            let mut sounds:Vec<SoundData> = vec![];
            self.playing_sounds.keys().for_each(|path| {
                if let Some(sound) = self.get_sound_by_path(path) {
                    sounds.push(SoundData {
                        source: path.clone(),
                        volume: sound.volume(),
                    });
                }
            });
            self.scenes[scene_index].sounds = sounds;
            info!("Scene {} saved", scene_index);
        } else {
            error!("Scene index out of bounds");
        }
    }



    //===== Serialization
    fn load_scene_collection(&mut self) -> Result<(), FileError> {
        let home_dir = my_home().map_err(|e| FileError::IoError(std::io::Error::new(std::io::ErrorKind::Unsupported, e)))?;
        let path = home_dir.ok_or_else(|| FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")))?
        .to_str()
        .unwrap()
        .to_string() + "/.config/serenIT/sounds.json";

        self.config_path = path.clone();
    
        self.read_from_file(&path)
    }

    fn read_from_file(&mut self, path: &str) -> Result<(), FileError> {
        let mut file = File::open(path)?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;

        //Get all scenes
        self.scenes = serde_json::from_str(&buff)?;

        if self.scenes.is_empty() {
            self.create_empty_scene();
        }
        self.scenes.sort_by(|a, b| a.name.cmp(&b.name));

        //Get first scene
        let first = self.scenes.get(0).cloned()
        .ok_or(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "No scene found")))?;

        for (i, s) in first.sounds.iter().enumerate() {
            if s.source.len() > 0 {
                info!("Loading from file: {}, with volume {}", s.source, s.volume);
                self.available_sounds.iter_mut()
                .find(|sound| sound.path() == s.source)
                .map(|sound| sound.set_volume(s.volume));

                self.set_sink_source(i, &s.source, s.volume);
            }
        }
        self.current_scene_index = 0;
        self.pause_all();
        Ok(())
    }

    fn write_scene_collection_to(&mut self, path: String) -> Result<(), FileError> {
        // Create/open the file
        let file_path = Path::new(&path);
        let parent_dir = file_path.parent().ok_or(FileError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "No parent directory found")))?;
        std::fs::create_dir_all(parent_dir)?;
        let mut f = File::create(path)?;

        // Serialize the struct
        self.save_current_scene();
        let serialized = serde_json::to_string(&self.scenes)?;

        info!("Saving to file: {}", serialized);

        // Write to file
        f.write_all(serialized.as_bytes())?;

        Ok(())
    }




    //====== Sounds initialization
    /// Load the sound collection from the SOUNDS array
    fn load_sound_collection(&mut self) {
        self.available_sounds.clear();
        SOUNDS.iter().for_each(|(path,_)| {
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

pub const SOUNDS: [(&str, std::io::Cursor<&[u8]>); 77] = [
    ("sounds/animals/birds.mp3", Cursor::new(include_bytes!("../sounds/animals/birds.mp3"))),
    ("sounds/animals/crickets.mp3", Cursor::new(include_bytes!("../sounds/animals/crickets.mp3"))),
    ("sounds/animals/crows.mp3", Cursor::new(include_bytes!("../sounds/animals/crows.mp3"))),
    ("sounds/animals/dog-barking.mp3", Cursor::new(include_bytes!("../sounds/animals/dog-barking.mp3"))),
    ("sounds/animals/frog.mp3", Cursor::new(include_bytes!("../sounds/animals/frog.mp3"))),
    ("sounds/animals/horse-galopp.mp3", Cursor::new(include_bytes!("../sounds/animals/horse-galopp.mp3"))),
    ("sounds/animals/owl.mp3", Cursor::new(include_bytes!("../sounds/animals/owl.mp3"))),
    ("sounds/animals/seagulls.mp3", Cursor::new(include_bytes!("../sounds/animals/seagulls.mp3"))),
    ("sounds/animals/whale.mp3", Cursor::new(include_bytes!("../sounds/animals/whale.mp3"))),
    ("sounds/animals/wolf.mp3", Cursor::new(include_bytes!("../sounds/animals/wolf.mp3"))),
    ("sounds/binaural/binaural-alpha.wav", Cursor::new(include_bytes!("../sounds/binaural/binaural-alpha.wav"))),
    ("sounds/binaural/binaural-beta.wav", Cursor::new(include_bytes!("../sounds/binaural/binaural-beta.wav"))),
    ("sounds/binaural/binaural-delta.wav", Cursor::new(include_bytes!("../sounds/binaural/binaural-delta.wav"))),
    ("sounds/binaural/binaural-gamma.wav", Cursor::new(include_bytes!("../sounds/binaural/binaural-gamma.wav"))),
    ("sounds/binaural/binaural-theta.wav", Cursor::new(include_bytes!("../sounds/binaural/binaural-theta.wav"))),
    ("sounds/nature/campfire.mp3", Cursor::new(include_bytes!("../sounds/nature/campfire.mp3"))),
    ("sounds/nature/droplets.mp3", Cursor::new(include_bytes!("../sounds/nature/droplets.mp3"))),
    ("sounds/nature/jungle.mp3", Cursor::new(include_bytes!("../sounds/nature/jungle.mp3"))),
    ("sounds/nature/river.mp3", Cursor::new(include_bytes!("../sounds/nature/river.mp3"))),
    ("sounds/nature/walk-in-snow.mp3", Cursor::new(include_bytes!("../sounds/nature/walk-in-snow.mp3"))),
    ("sounds/nature/walk-on-leaves.mp3", Cursor::new(include_bytes!("../sounds/nature/walk-on-leaves.mp3"))),
    ("sounds/nature/waterfall.mp3", Cursor::new(include_bytes!("../sounds/nature/waterfall.mp3"))),
    ("sounds/nature/waves.mp3", Cursor::new(include_bytes!("../sounds/nature/waves.mp3"))),
    ("sounds/nature/wind.mp3", Cursor::new(include_bytes!("../sounds/nature/wind.mp3"))),
    ("sounds/nature/wind-in-trees.mp3", Cursor::new(include_bytes!("../sounds/nature/wind-in-trees.mp3"))),
    ("sounds/noise/brown-noise.wav", Cursor::new(include_bytes!("../sounds/noise/brown-noise.wav"))),
    ("sounds/noise/pink-noise.wav", Cursor::new(include_bytes!("../sounds/noise/pink-noise.wav"))),
    ("sounds/noise/white-noise.wav", Cursor::new(include_bytes!("../sounds/noise/white-noise.wav"))),
    ("sounds/places/airport.mp3", Cursor::new(include_bytes!("../sounds/places/airport.mp3"))),
    ("sounds/places/cafe.mp3", Cursor::new(include_bytes!("../sounds/places/cafe.mp3"))),
    ("sounds/places/carousel.mp3", Cursor::new(include_bytes!("../sounds/places/carousel.mp3"))),
    ("sounds/places/church.mp3", Cursor::new(include_bytes!("../sounds/places/church.mp3"))),
    ("sounds/places/construction-site.mp3", Cursor::new(include_bytes!("../sounds/places/construction-site.mp3"))),
    ("sounds/places/crowded-bar.mp3", Cursor::new(include_bytes!("../sounds/places/crowded-bar.mp3"))),
    ("sounds/places/laboratory.mp3", Cursor::new(include_bytes!("../sounds/places/laboratory.mp3"))),
    ("sounds/places/laundry-room.mp3", Cursor::new(include_bytes!("../sounds/places/laundry-room.mp3"))),
    ("sounds/places/night-village.mp3", Cursor::new(include_bytes!("../sounds/places/night-village.mp3"))),
    ("sounds/places/office.mp3", Cursor::new(include_bytes!("../sounds/places/office.mp3"))),
    ("sounds/places/subway-station.mp3", Cursor::new(include_bytes!("../sounds/places/subway-station.mp3"))),
    ("sounds/places/supermarket.mp3", Cursor::new(include_bytes!("../sounds/places/supermarket.mp3"))),
    ("sounds/places/temple.mp3", Cursor::new(include_bytes!("../sounds/places/temple.mp3"))),
    ("sounds/places/underwater.mp3", Cursor::new(include_bytes!("../sounds/places/underwater.mp3"))),
    ("sounds/rain/heavy-rain.mp3", Cursor::new(include_bytes!("../sounds/rain/heavy-rain.mp3"))),
    ("sounds/rain/light-rain.mp3", Cursor::new(include_bytes!("../sounds/rain/light-rain.mp3"))),
    ("sounds/rain/rain-on-leaves.mp3", Cursor::new(include_bytes!("../sounds/rain/rain-on-leaves.mp3"))),
    ("sounds/rain/rain-on-tent.mp3", Cursor::new(include_bytes!("../sounds/rain/rain-on-tent.mp3"))),
    ("sounds/rain/rain-on-umbrella.mp3", Cursor::new(include_bytes!("../sounds/rain/rain-on-umbrella.mp3"))),
    ("sounds/rain/rain-on-window.mp3", Cursor::new(include_bytes!("../sounds/rain/rain-on-window.mp3"))),
    ("sounds/rain/thunder.mp3", Cursor::new(include_bytes!("../sounds/rain/thunder.mp3"))),
    ("sounds/things/boiling-water.mp3", Cursor::new(include_bytes!("../sounds/things/boiling-water.mp3"))),
    ("sounds/things/bubbles.mp3", Cursor::new(include_bytes!("../sounds/things/bubbles.mp3"))),
    ("sounds/things/ceiling-fan.mp3", Cursor::new(include_bytes!("../sounds/things/ceiling-fan.mp3"))),
    ("sounds/things/clock.mp3", Cursor::new(include_bytes!("../sounds/things/clock.mp3"))),
    ("sounds/things/dryer.mp3", Cursor::new(include_bytes!("../sounds/things/dryer.mp3"))),
    ("sounds/things/keyboard.mp3", Cursor::new(include_bytes!("../sounds/things/keyboard.mp3"))),
    ("sounds/things/morse-code.mp3", Cursor::new(include_bytes!("../sounds/things/morse-code.mp3"))),
    ("sounds/things/paper.mp3", Cursor::new(include_bytes!("../sounds/things/paper.mp3"))),
    ("sounds/things/slide-projector.mp3", Cursor::new(include_bytes!("../sounds/things/slide-projector.mp3"))),
    ("sounds/things/singing-bowl.mp3", Cursor::new(include_bytes!("../sounds/things/singing-bowl.mp3"))),
    ("sounds/things/tuning-radio.mp3", Cursor::new(include_bytes!("../sounds/things/tuning-radio.mp3"))),
    ("sounds/things/typewriter.mp3", Cursor::new(include_bytes!("../sounds/things/typewriter.mp3"))),
    ("sounds/things/washing-machine.mp3", Cursor::new(include_bytes!("../sounds/things/washing-machine.mp3"))),
    ("sounds/things/wind-chimes.mp3", Cursor::new(include_bytes!("../sounds/things/wind-chimes.mp3"))),
    ("sounds/transport/airplane.mp3", Cursor::new(include_bytes!("../sounds/transport/airplane.mp3"))),
    ("sounds/transport/inside-a-train.mp3", Cursor::new(include_bytes!("../sounds/transport/inside-a-train.mp3"))),
    ("sounds/transport/rowing-boat.mp3", Cursor::new(include_bytes!("../sounds/transport/rowing-boat.mp3"))),
    ("sounds/transport/sailboat.mp3", Cursor::new(include_bytes!("../sounds/transport/sailboat.mp3"))),
    ("sounds/transport/submarine.mp3", Cursor::new(include_bytes!("../sounds/transport/submarine.mp3"))),
    ("sounds/transport/train.mp3", Cursor::new(include_bytes!("../sounds/transport/train.mp3"))),
    ("sounds/urban/ambulance-siren.mp3", Cursor::new(include_bytes!("../sounds/urban/ambulance-siren.mp3"))),
    ("sounds/urban/busy-street.mp3", Cursor::new(include_bytes!("../sounds/urban/busy-street.mp3"))),
    ("sounds/urban/crowd.mp3", Cursor::new(include_bytes!("../sounds/urban/crowd.mp3"))),
    ("sounds/urban/fireworks.mp3", Cursor::new(include_bytes!("../sounds/urban/fireworks.mp3"))),
    ("sounds/urban/highway.mp3", Cursor::new(include_bytes!("../sounds/urban/highway.mp3"))),
    ("sounds/urban/road.mp3", Cursor::new(include_bytes!("../sounds/urban/road.mp3"))),
    ("sounds/urban/traffic.mp3", Cursor::new(include_bytes!("../sounds/urban/traffic.mp3"))),
    ("sounds/alarm.mp3", Cursor::new(include_bytes!("../sounds/alarm.mp3"))),
];