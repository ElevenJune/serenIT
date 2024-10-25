use cli_log::*;

use crate::sink_handle::SinkHandle;
use crate::sound::Sound;

const MAX_SOUNDS: usize = 3;

pub struct SoundManager {
    available_sounds: Vec<Sound>,
    sinks: Vec<SinkHandle>,
}

pub enum SoundManagerError {
    NoAvailableSound,
    AlreadyPlaying,
    SoundDoesNotExists,
    OtherError,
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
        };
        sm.load_available_sounds();
        sm
    }

    pub fn get_sound_list(&self) -> &Vec<Sound> {
        &self.available_sounds
    }

    pub fn is_sound_playing(&self, name: &str) -> bool {
        match self.get_sound_by_name(name) {
            Some(s) => self.is_source_playing(s.path()),
            None => false,
        }
    }

    pub fn get_sound_name_by_index(&self, index: usize) -> &str {
        self.available_sounds[index].name()
    }

    pub fn is_source_playing(&self, source: &str) -> bool {
        self.sinks
            .iter()
            .any(|s| s.get_source() == source && s.is_playing())
    }

    pub fn toggle_sound(&mut self, name: &str) -> Result<(), SoundManagerError> {
        let res = if self.is_sound_playing(name) {
            self.remove_sound(name)
        } else {
            self.add_sound(name)
        };
        self.update_all();
        self.order_sinks_by_playing();
        res
    }

    pub fn add_sound(&mut self, name: &str) -> Result<(), SoundManagerError> {
        self.update_all();
        let sound = match self.get_sound_by_name(name) {
            Some(s) => s,
            None => return Err(SoundManagerError::SoundDoesNotExists),
        };
        if self.is_sound_playing(name) {
            return Err(SoundManagerError::AlreadyPlaying);
        }
        let path = &sound.path().to_string().clone();
        let volume = sound.volume();
        let res = match self.find_available() {
            Some(i) => {
                info!("Playing {} with volume {} on sink {}",path, volume, i);
                self.sinks[i].set_volume(volume);
                self.sinks[i].set_source(path);
                self.sinks[i].play();
                Ok(())
            }
            None => {
                self.overwrite_last(path, volume);
                warn!("{} overwrote last sound", path);
                Err(SoundManagerError::NoAvailableSound)
            }
        };
        self.update_all();
        res
    }

    pub fn get_sound_by_name(&self, name: &str) -> Option<&Sound> {
        self.available_sounds.iter().find(|s| s.name() == name)
    }

    pub fn order_sinks_by_playing(&mut self) {
        self.sinks
            .sort_by(|a, b| b.is_playing().cmp(&a.is_playing()));
    }

    pub fn remove_sound(&mut self, name: &str) -> Result<(), SoundManagerError> {
        let sound = match self.get_sound_by_name(name) {
            Some(s) => s,
            None => return Err(SoundManagerError::SoundDoesNotExists),
        };
        if !self.is_sound_playing(name) {
            return Err(SoundManagerError::AlreadyPlaying);
        }
        let path = &sound.path().to_string();
        self.sinks
            .iter_mut()
            .find(|s| s.get_source() == path && s.is_playing())
            .map(|s| s.stop());
        Ok(())
    }

    pub fn adjust_volume(&mut self, name: &str, volume_offset: f32) {
        //Store new volume
        match self.available_sounds.iter_mut().find(|s| s.name() == name) {
            Some(s) => {
                let mut new_volume = s.volume().clone() + volume_offset;
                if new_volume > 2. {
                    new_volume = 2.;
                } else if new_volume < 0. {
                    new_volume = 0.;
                }
                s.set_volume(new_volume);
                self.sinks.iter_mut().for_each(|sh| {
                    if sh.get_source() == s.path() {
                        sh.set_volume(new_volume);
                    }
                });
            }
            None => {}
        }
        //Update volume
    }

    pub fn get_volume(&self) -> f32 {
        self.sinks[0].volume()
    }

    pub fn sinks(&self) -> &Vec<SinkHandle> {
        &self.sinks
    }

    fn overwrite_last(&mut self, source: &String, volume: f32) {
        match self.sinks.first_mut() {
            Some(s) => {
                s.stop();
                s.set_volume(volume);
                s.set_source(source);
                s.play();
            }
            None => {}
        }
    }

    fn find_available(&self) -> Option<usize> {
        self.sinks
            .iter()
            .enumerate()
            .find(|(_, s)| !s.is_playing())
            .map(|(i, _)| i)
    }

    pub fn update_all(&mut self) {
        self.sinks
            .iter_mut()
            .for_each(|s: &mut SinkHandle| s.update());
    }

    fn load_available_sounds(&mut self) {
        self.available_sounds.clear();
        let params: Vec<&str> = vec![
            // sounds/
            "./sounds/alarm.mp3",
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
        ];
        params.iter().for_each(|path| {
            let filename = path.split('/').last().unwrap();
            let volume = if filename.contains("binaural") {
                0.2
            } else {
                1.
            };
            self.available_sounds
                .push(Sound::new(filename, path, volume));
        });
    }
}
