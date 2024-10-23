use rodio::source::{SineWave, Source};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use crate::sound::Sound;

pub struct SoundManager{
    sounds : Vec<Sound>
}