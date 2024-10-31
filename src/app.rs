use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{AMBER, TEAL},
        Color, Modifier, Style, Stylize,
    },
    symbols::{self},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, LineGauge, List, ListItem, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    widgets::ListState,
    DefaultTerminal,
};
use std::sync::Arc;

use crate::sound_manager::SoundManager;
use cli_log::*;
use color_eyre::{owo_colors::OwoColorize, Result};

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
            KeyCode::Char('g') | KeyCode::Home => {}
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

const TODO_HEADER_STYLE: Style = Style::new()
    .fg(TEAL.c100)
    .bg(TEAL.c800)
    .add_modifier(Modifier::BOLD);
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
const MIXER_BORDERS_STYLE: Style = Style::new()
    .fg(TEAL.c100)
    .bg(TEAL.c500)
    .add_modifier(Modifier::BOLD);
const EDIT_ROW_COLOR: Color = AMBER.c700;
const EDIT_VALUE_COLOR: Color = AMBER.c500;
const EDIT_STYLE: Style = Style::new()
    .bg(EDIT_ROW_COLOR)
    .add_modifier(Modifier::BOLD)
    .fg(AMBER.c100);
const EDIT_VALUE_STYLE: Style = Style::new()
    .bg(EDIT_VALUE_COLOR)
    .add_modifier(Modifier::BOLD)
    .fg(AMBER.c100);
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = TEAL.c200;
const TEXT_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);

impl App {
    //Renders header
    fn render_header(area: Rect, buf: &mut Buffer) {
        Arc::new(
            Paragraph::new("Ambient Sound Player")
                .bold()
                .centered()
                .bg(TEAL.c500)
                .render(area, buf),
        );
    }

    //Renders footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let text = "Add/Remove the selected sound with Enter\n\
            -/+ to adjust the volume, ctrl+/- to adjust the master volume\n\
            's' to save, 'c' to swicth category, 'q' to quit";
        Paragraph::new(text)
            .centered()
            .bg(AMBER.c100)
            .fg(EDIT_ROW_COLOR)
            .bold()
            .render(area, buf);
    }

    //Renders left list
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        //Category
        let categories = self.sound_manager.categories();
        let category_text = match self.category {
            Some(i) => format!(
                "{}/{} {}",
                i + 1,
                categories.len(),
                categories[i].to_uppercase()
            ),
            None => "All".to_string(),
        };
        let category_line = Line::styled(
            "Category: ".to_string() + &category_text,
            TODO_HEADER_STYLE.fg(if self.category.is_some() {
                AMBER.c100
            } else {
                TEAL.c100
            }),
        )
        .centered();

        // Sounds
        let items: Vec<ListItem> = self
            .sound_manager
            .get_sound_list()
            .iter()
            .filter(|s| {
                if let Some(c) = self.category {
                    s.category() == self.sound_manager.categories()[c]
                } else {
                    true
                }
            })
            .enumerate()
            .map(|(i, s)| {
                let color = alternate_colors(i);
                let displayed_name = if self.category.is_none() {
                    format!("[{}] {}", s.category().to_uppercase(), s.name())
                } else {
                    s.name().to_string()
                };
                let mut item = ListItem::from(displayed_name).bg(color);
                if self.sound_manager.is_sound_playing(s.path()) {
                    item = item.add_modifier(Modifier::BOLD).fg(AMBER.c100);
                }
                item
            })
            .collect();

        //Render
        let block = Block::new()
            .title(Line::raw("Sounds List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(TEAL.c800);

        let [cat_layout, list_layout] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(block.inner(area));

        let selected_style = SELECTED_STYLE;
        let symbol = " => ";

        let list = List::new(items)
            .highlight_style(selected_style)
            .highlight_symbol(symbol)
            .highlight_spacing(HighlightSpacing::Always);

        block.render(area, buf);
        Paragraph::new(category_line).render(cat_layout, buf);
        StatefulWidget::render(list, list_layout, buf, &mut self.state);
    }

    fn render_current_sounds(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::styled("Mixer",TODO_HEADER_STYLE).centered())
            .borders(Borders::LEFT)
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .border_style(MIXER_BORDERS_STYLE)
            .bg(NORMAL_ROW_BG);

        let sounds = self.sound_manager.playing_sounds();
        let mut constr: Vec<Constraint> = vec![];
        for _i in 0..sounds.len() {
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
        }
        constr.push(Constraint::Fill(1));

        let layouts = Layout::vertical(constr).split(block.inner(area));

        block.render(area, buf);

        sounds.iter().enumerate().for_each(|(i, (p, _))| {
            let path = p.as_str();
            if !self.sound_manager.is_sound_playing(path) {
                return;
            }

            let volume = match self.sound_manager.get_sound_by_path(path) {
                Some(sound) => sound.volume(),
                None => 0.0,
            };

            Paragraph::new(path)
                .wrap(Wrap { trim: false })
                .render(layouts[3 * i], buf);

            LineGauge::default()
                .filled_style(Style::default().fg(TEAL.c100))
                .unfilled_style(Style::default().fg(TEAL.c800))
                .ratio(volume.into())
                .line_set(symbols::line::THICK)
                .render(layouts[3 * i + 1], buf);
        });
    }
}

//Renders whole app
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let footer_length = if true { 3 } else { 2 };
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(footer_length),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_current_sounds(item_area, buf);
    }
}

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
