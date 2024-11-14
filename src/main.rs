use app::App;
use color_eyre::Result;
use sound_manager::SoundManager;

mod app;
mod sink_handle;
mod sound;
mod sound_manager;
mod ui;

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
// - put sounds inside the exe
// - tab widget
// - Multiple config files to choose from
// - switch the volume directly in the mixer DONE
// - Favorite sounds category
// - Better UI DONE
// - small animation ?
// - tab with logs
