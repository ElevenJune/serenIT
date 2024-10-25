use app::App;
use color_eyre::Result;
use sound_manager::SoundManager;

mod app;
mod sink_handle;
mod sound;
mod sound_manager;

fn main() -> Result<()> {
    cli_log::init_cli_log!();
    let app = App::new(SoundManager::new());
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
