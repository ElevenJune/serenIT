use app::App;
use color_eyre::Result;

mod sound;
mod app;

use crate::sound::Sound;

fn main() -> Result<()> {
    let mut sounds: Vec<Sound> = vec![];
    let params: Vec<(String, f32)> = vec![
        //("./sounds/nature/waves.mp3".to_string(), 0.5),
        //("./sounds/things/wind-chimes.mp3".to_string(), 0.5),
        ("./sounds/binaural/binaural-alpha.wav".to_string(), 0.01),
    ];
/*
    // Create Sound instances with separate sinks for each sound
    for (source, volume) in params.iter() {
        sounds.push(Sound::new(&source, *volume));
    } 

    // Code à exécuter dans le nouveau thread
    sounds[0].play();
    //std::thread::sleep(std::time::Duration::from_secs(1));
    //sounds[1].play();
    //std::thread::sleep(std::time::Duration::from_secs(1));
    //sounds[2].play();
/
    let mut i = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("{}", i);
        for j in 0..sounds.len() {
            sounds[j].update();
            //println!("Sound {} playing ? {}",j,sounds[j].is_playing());
        }
        if i == 19 {
            sounds[0].set_source(&"./sounds/nature/campfire.mp3".to_string());
            sounds[0].play();
        }
        i += 1;
        if i == 100 {
            return;
        }
    }*/

    let app = App::new(Sound::new(&"./sounds/binaural/binaural-alpha.wav".to_string(), 0.01));
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = app.run(terminal);
    ratatui::restore();
    app_result
}

//Todo
// - Add a way to stop the sounds
// - Add a way to change the volume of the sounds
// - Load a new sound
// - delete a sink to create a new one
