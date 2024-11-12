use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ListState,
    DefaultTerminal,
};

use crate::sound_manager::SoundManager;
use cli_log::*;
use color_eyre::Result;

pub struct App {
    exit: bool,
    state: ListState,
    sound_manager: SoundManager,
    category: Option<usize>,
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    pub fn new(sound_manager: SoundManager) -> Self {
        App {
            exit: false,
            state: ListState::default(),
            sound_manager,
            category: None,
        }
    }

    //----Getters
    pub fn get_state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn get_selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn get_category(&self) -> Option<usize> {
        self.category
    }

    pub fn get_sound_manager(&self) -> &SoundManager {
        &self.sound_manager
    }

    //----Event handling

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.change_volume(-0.02, ctrl_pressed),
            KeyCode::Char('i') | KeyCode::Right => self.change_volume(0.02, ctrl_pressed),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char(' ') => {self.sound_manager.toggle_pause_play();},
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('c') => self.swicth_category(),
            KeyCode::Char('s') => {
                let _ = self.sound_manager.save();
            }
            KeyCode::Enter => self.toogle_selected_sound(),
            _ => {}
        }
    }

    fn _select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn _select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        /*if let Some(index) = self.list.items().len().checked_sub(1) {
            self.state.select(Some(index));
        }*/
    }

    fn swicth_category(&mut self) {
        let categories = self.sound_manager.categories();
        self.category = match self.category {
            None => Some(0),
            Some(i) => {
                if i + 1 == categories.len() {
                    None
                } else {
                    Some(i + 1)
                }
            }
        }
    }
    fn change_volume(&mut self, volume_offset: f32, master: bool) {
        if master {
            self.change_master_volume(volume_offset);
        } else {
            self.change_sound_volume(volume_offset);
        }
    }

    fn change_sound_volume(&mut self, volume_offset: f32) {
        if let Some(index) = self.state.selected() {
            let path = self
                .sound_manager
                .get_sound_path_by_index_and_category(index, self.category)
                .to_string();
            self.sound_manager.adjust_sound_volume(&path, volume_offset);
        }
    }

    fn change_master_volume(&mut self, volume_offset: f32) {
        self.sound_manager.adjust_master_volume(volume_offset);
    }

    fn toogle_selected_sound(&mut self) {
        if let Some(index) = self.state.selected() {
            let path = self
                .sound_manager
                .get_sound_path_by_index_and_category(index, self.category)
                .to_string();
            info!("Toggling sound: {}", path);
            let _ = self.sound_manager.toggle_sound(&path);
        }
    }
}
