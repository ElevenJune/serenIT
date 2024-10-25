use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Span,
    style::{
        palette::tailwind::{AMBER, TEAL},
        Color, Modifier, Style, Stylize,
    },
    symbols::{self},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, StatefulWidget,
        Widget, Wrap, LineGauge
    },
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    widgets::ListState,
    DefaultTerminal,
};
use std::io::{stdout, Stdout, Write};
use std::sync::Arc;

use crate::sound_manager::{self, SoundManager};
use color_eyre::Result;

pub struct App {
    exit: bool,
    state: ListState,
    sound_manager: SoundManager,
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
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.change_sound_volume(-0.02),
            KeyCode::Char('i') | KeyCode::Right => self.change_sound_volume(0.02),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => {}
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Enter => self.toogle_selected_sound(),
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        /*if let Some(index) = self.list.items().len().checked_sub(1) {
            self.state.select(Some(index));
        }*/
    }

    fn change_sound_volume(&mut self, volume_offset: f32) {
        if let Some(index) = self.state.selected() {
            let (selected_source, _) = &self.sound_manager.get_sound_list()[index].clone();
            self.sound_manager
                .adjust_volume(selected_source, volume_offset);
        }
    }

    fn toogle_selected_sound(&mut self) {
        self.sound_manager.update_all();
        if let Some(index) = self.state.selected() {
            let (selected_source, _) = &self.sound_manager.get_sound_list()[index].clone();
            if self.sound_manager.is_source_playing(&selected_source) {
                self.sound_manager.remove_sound(selected_source);
            } else {
                let _ = self.sound_manager.add_sound(selected_source);
            }
        }
    }
}

const TODO_HEADER_STYLE: Style = Style::new().fg(TEAL.c100).bg(TEAL.c800);
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
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
            Paragraph::new("Todo List Application")
                .bold()
                .centered()
                .bg(TEAL.c500)
                .render(area, buf),
        );
    }

    //Renders footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let text = if true {
            "[Edit Mode]\nSave with Enter, Cancel with Esc\n-/+ to change priority, type to change name"
        } else {
            "Use ↓↑ to move, ← to unselect, → to change status\n'a' to add a task. 'Delete' to remove a task"
        };
        Paragraph::new(text)
            .centered()
            .bg(AMBER.c100)
            .fg(EDIT_ROW_COLOR)
            .bold()
            .render(area, buf);
    }

    //Renders left list
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Sounds List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .sound_manager
            .get_sound_list()
            .iter()
            .enumerate()
            .map(|(i, (source, name))| {
                let color = alternate_colors(i);
                let displayed_name = name.clone();
                let mut item = ListItem::from(displayed_name).bg(color);
                if self.sound_manager.is_source_playing(source) {
                    item = item.add_modifier(Modifier::BOLD).fg(AMBER.c100);
                }
                item
            })
            .collect();

        let mut selected_style = SELECTED_STYLE;
        let mut symbol = " => ";

        let list = List::new(items)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol(symbol)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.get_state());
    }

    fn render_current_sounds(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Mixer").centered())
            .borders(Borders::all())
            .border_set(symbols::border::EMPTY)
            .border_style(EDIT_STYLE)
            .bg(NORMAL_ROW_BG);

        let sounds = self.sound_manager.sounds();
        let mut constr : Vec<Constraint> = vec![];
        for _i in 0..sounds.len(){
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
        }
        constr.push(Constraint::Fill(1));
        const SIZE :usize = 4*3+1;

        let layouts = Layout::vertical(constr)
        .split(block.inner(area));

        //let mut linegauges: Vec<LineGauge> = Vec::new();
        /*let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(sounds.iter().map(|_| Constraint::Fill(1)).collect::<Vec<_>>())
        .split(area);*/

        block.render(area, buf);

        sounds.iter().enumerate().for_each(|(i,s)| {
            if !s.is_playing() {return;}

            Paragraph::new(s.get_source().as_str())
            .wrap(Wrap { trim: false })
            .render(layouts[3*i], buf);

            LineGauge::default()
            .filled_style(Style::default().fg(Color::Blue))
            .unfilled_style(Style::default().fg(Color::Red))
            .ratio(s.volume().into())
            .line_set(symbols::line::THICK)
            .render(layouts[3*i+1], buf);
        });
    }

    //Renders selected task (right)
    /*fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let mut text: Vec<Line<'_>> = vec![];
        let border_style = if self.is_edit_mode() { EDIT_STYLE } else { TODO_HEADER_STYLE };

        match &self.get_selected() {
            Some(i) => {
                let task = self.get_list().task(*i);
                let style = if self.is_edit_mode() { EDIT_VALUE_STYLE } else { TEXT_STYLE };

                let mut name_line = vec!["Name : ".red()];

                let mut priority_line = vec!["Priority : ".red()];

                let state_line = vec![
                    "Done : ".red(),
                    Span::styled(format!("{}", task.done), TEXT_STYLE),
                ];

                if self.is_edit_mode() {
                    name_line.push(Span::styled(self.get_edit_name(), style));
                    priority_line.push(Span::styled(format!("{}", self.get_edit_priority()), style));
                    name_line.push("_".fg(EDIT_VALUE_COLOR).add_modifier(Modifier::BOLD));
                    priority_line.push(" (-/+)".fg(EDIT_VALUE_COLOR).bold());
                } else {
                    name_line.push(Span::styled(&task.name, style));
                    priority_line.push(Span::styled(format!("{}", task.priority), TEXT_STYLE));
                }

                text.push(Line::from(name_line));
                text.push(Line::from(priority_line));
                text.push(Line::from(state_line));
            }
            None => {
                text.push(Line::styled("Select a task", Style::new().gray().italic()));
            }
        }

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Task Information").centered())
            .borders(Borders::all())
            .border_set(symbols::border::EMPTY)
            .border_style(border_style)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }*/
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

        let info_weight = if true { 2 } else { 1 };
        let [list_area, item_area] = Layout::horizontal([
            Constraint::Fill(3 - info_weight),
            Constraint::Fill(info_weight),
        ])
        .areas(main_area);

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

#[cfg(test)]
mod tests {
    use super::*;
}
