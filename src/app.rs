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
    mixer_index: Option<usize>,
    mixer_mode: bool,
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
            mixer_index: None,
            mixer_mode: false,
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

    pub fn get_mixer_index(&self) -> Option<usize> {
        self.mixer_index
    }

    pub fn get_mixer_mode(&self) -> bool {
        self.mixer_mode
    }

    //----Event handling

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.arrow_pressed(true, ctrl_pressed),
            KeyCode::Char('i') | KeyCode::Right => self.arrow_pressed(false, ctrl_pressed),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => self.toogle_selected_sound(),
            KeyCode::Tab => self.switch_menu(),
            KeyCode::Char(' ') => self.sound_manager.toggle_pause_play(),
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('s') => {
                let _ = self.sound_manager.save();
            }
            _ => {}
        }
    }

    fn _select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        if self.mixer_mode {
            match self.mixer_index {
                Some(index) => {
                    self.set_mixer_index(index + 1);
                }
                None => {
                    self.mixer_index = Some(0);
                }
            }
        } else {
            self.state.select_next();
        }
    }

    fn set_mixer_index(&mut self, index: usize) {
        let len = self.sound_manager.playing_sounds().len();
        if len == 0 {
            self.mixer_index = None;
        } else if index >= len {
            self.mixer_index = Some(len - 1);
        } else {
            self.mixer_index = Some(index);
        }
    }

    fn select_previous(&mut self) {
        if self.mixer_mode {
            match self.mixer_index {
                Some(index) => {
                    self.set_mixer_index(index.checked_sub(1).unwrap_or(0));
                }
                None => {
                    self.mixer_index = Some(0);
                }
            }
        } else {
            self.state.select_previous();
        }
    }

    fn _select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        if !self.mixer_mode {
            if let Some(index) = self.sound_manager.get_sound_list().len().checked_sub(1) {
                self.state.select(Some(index));
            }
        } else {
            if let Some(index) = self.sound_manager.playing_sounds().len().checked_sub(1) {
                self.mixer_index = Some(index);
            }
        }
    }

    fn switch_menu(&mut self) {
        self.mixer_mode = !self.mixer_mode;
        if self.mixer_index.is_none() {
            self.mixer_index = if self.get_sound_manager().playing_sounds().is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn arrow_pressed(&mut self, left: bool, ctrl_pressed: bool) {
        let volume = if left { -0.02 } else { 0.02 };
        if ctrl_pressed {
            self.change_volume(volume, true);
        } else if self.mixer_mode {
            self.change_volume(volume, false);
        } else {
            self.switch_category(left);
        }
    }

    fn switch_category(&mut self, backward: bool) {
        let categories = self.sound_manager.categories();
        let len = categories.len();
        self.category = match self.category {
            None => Some(if backward { len - 1 } else { 0 }),
            Some(i) => {
                if (i + 1 == len && !backward) || (i == 0 && backward) {
                    None
                } else {
                    Some(if backward { i - 1 } else { i + 1 })
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
        if let Some(path) = self.get_mixer_selected_path() {
            self.sound_manager.adjust_sound_volume(&path, volume_offset);
        }
    }

    fn change_master_volume(&mut self, volume_offset: f32) {
        self.sound_manager.adjust_master_volume(volume_offset);
    }

    fn toogle_selected_sound(&mut self) {
        if !self.get_mixer_mode() {
            if let Some(path) = self.get_sound_selected_path() {
                info!("Toggling sound: {}", path);
                let _ = self.sound_manager.toggle_sound(&path);
                self.mixer_index = None;
            }
        } else {
            if let Some(path) = self.get_mixer_selected_path() {
                info!("Toggling sound: {}", path);
                let _ = self.sound_manager.toggle_sound(&path);
                if self.mixer_index.is_some()
                    && self.mixer_index.unwrap() >= self.sound_manager.playing_sounds().len()
                {
                    self.select_next();
                }
            }
        }
    }

    fn get_mixer_selected_path(&self) -> Option<String> {
        self.mixer_index
            .and_then(|index| self.sound_manager.playing_sounds().keys().nth(index))
            .map(|path| path.to_string())
    }

    fn get_sound_selected_path(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|index| {
                self.sound_manager
                    .get_sound_path_by_index_and_category(index, self.category)
            })
            .map(|path| path.to_string())
    }
}
