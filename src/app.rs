use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ListState,
    DefaultTerminal,
};

use crate::sound_manager::SoundManager;
use cli_log::*;
use color_eyre::Result;

pub struct App {
    sound_manager: SoundManager,
    exit: bool,
    //Sound tab
    sound_list_tab: bool,
    category: Option<usize>,
    sound_state: ListState,
    //Scene tab
    scene_state: ListState,
    //Mixer
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
        let mut app =App {
            exit: false,
            sound_state: ListState::default(),
            scene_state: ListState::default(),
            sound_manager,
            category: None,
            mixer_index: None,
            mixer_mode: false,
            sound_list_tab: true
        };
        app.scene_state.select(Some(app.sound_manager.get_current_scene_index()));
        app
    }

    //----Getters
    pub fn get_sound_list_state(&mut self) -> &mut ListState {
        &mut self.sound_state
    }

    pub fn get_scene_list_state(&mut self) -> &mut ListState {
        &mut self.scene_state
    }

    pub fn get_selected(&self) -> Option<usize> {
        self.sound_state.selected()
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

    pub fn get_sound_list_tab(&self) -> bool {
        self.sound_list_tab
    }

    pub fn get_mixer_selected_path(&self) -> Option<String> {
        self.mixer_index
            .and_then(|index| self.sound_manager.playing_sounds().keys().nth(index))
            .map(|path| path.to_string())
    }

    pub fn get_sound_selected_path(&self) -> Option<String> {
        self.sound_state
            .selected()
            .and_then(|index| {
                self.sound_manager
                    .get_sound_path_by_index_and_category(index, self.category)
            })
            .map(|path| path.to_string())
    }

    pub fn get_scene_selected_index(&self) -> Option<usize> {
        self.scene_state
            .selected()
    }

    //----Event handling

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            //Navigation
            KeyCode::Char('h') | KeyCode::Left => self.arrow_pressed(true, ctrl_pressed),
            KeyCode::Char('i') | KeyCode::Right => self.arrow_pressed(false, ctrl_pressed),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Tab => self.switch_input_tab(),
            KeyCode::Char('m') => self.switch_menu(),
            //Actions
            KeyCode::Enter => self.toogle_selected_sound(),
            KeyCode::Char(' ') => self.sound_manager.toggle_pause_play(),
            KeyCode::Char('n') => self.sound_manager.create_empty_scene(),
            KeyCode::Char('d') => self.delete_scene(),
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('s') => {let _ = self.sound_manager.save();},
            _ => {}
        }
    }

    fn _select_none(&mut self) {
        self.sound_state.select(None);
    }

    fn select_next(&mut self) {
        if self.mixer_mode {
            self.mixer_index = self.mixer_index.map_or(Some(0),
             |i| {
                let len = self.sound_manager.playing_sounds().len();
                if len == 0 {None}
                else if i >= len-1 {Some(len-1)}
                else {Some(i + 1)}
                });
        } else {
            match self.sound_list_tab {
                true => self.sound_state.select_next(),
                false => self.scene_state.select_next()
            }
        }
    }

    fn select_previous(&mut self) {
        if self.mixer_mode {
            self.mixer_index = self.mixer_index.map_or(Some(0),
            |i| i.checked_sub(1).unwrap_or(0).into());
        } else {
            match self.sound_list_tab {
                true => self.sound_state.select_previous(),
                false => self.scene_state.select_previous()
            }
        }
    }

    fn select_first(&mut self) {
        match self.sound_list_tab {
            true => self.sound_state.select_first(),
            false => self.scene_state.select_first()
        }
    }

    fn select_last(&mut self) {
        match self.mixer_mode {
            false => {
                let list_state = match self.sound_list_tab {
                    true => &mut self.sound_state,
                    false => &mut self.scene_state
                };
                let last_index = match self.sound_list_tab{
                    true => self.sound_manager.get_sound_list().len().checked_sub(1),
                    false => self.sound_manager.get_scene_collection().len().checked_sub(1)
                };
                if let Some(index) = last_index {
                    list_state.select(Some(index));
                }
            }
            true => {
                let last_index = self.sound_manager.playing_sounds().len().checked_sub(1);
                if let Some(index) = last_index {
                    self.mixer_index = Some(index);
                }
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

    pub fn switch_input_tab(&mut self) {
        if !self.get_mixer_mode() {
            self.sound_list_tab = !self.sound_list_tab;
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

    fn delete_scene(&mut self){
        if let Some(n) = self.scene_state.selected() {
            self.sound_manager.delete_scene(n);
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
        };
        self.select_first();
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
            if self.get_sound_list_tab(){
                if let Some(path) = self.get_sound_selected_path() {
                    info!("Toggling sound: {}", path);
                    let _ = self.sound_manager.toggle_sound(&path);
                    self.mixer_index = None;
                }
            }else{
                if let Some(index) = self.get_scene_selected_index() {
                    self.sound_manager.save_current_scene();
                    let _ = self.sound_manager.play_scene(index);
                    self.mixer_index = None;
                }
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

    fn _set_mixer_index(&mut self, index: usize) {
        let len = self.sound_manager.playing_sounds().len();
        if len == 0 {
            self.mixer_index = None;
        } else if index >= len {
            self.mixer_index = Some(len - 1);
        } else {
            self.mixer_index = Some(index);
        }
    }
}
