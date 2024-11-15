//use color_eyre::owo_colors::OwoColorize;
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
        Block, Borders, HighlightSpacing, Gauge, List, ListItem, Paragraph, StatefulWidget,
        Widget, Wrap,
    },
};
use cli_log::*;
use std::sync::Arc;
use crate::App;

const LIGHT_COLOR : Color = TEAL.c100;
const FOCUS_COLOR: Color = AMBER.c300;
const FOCUS_UNSELECTED_COLOR: Color = TEAL.c400;
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
const YELLOW: Color = AMBER.c100;

const HEADER_STYLE: Style = Style::new()
    .fg(LIGHT_COLOR)
    .bg(TEAL.c800)
    .add_modifier(Modifier::BOLD);
const BORDER_STYLE_NONE: symbols::border::Set = symbols::border::EMPTY;
const BORDER_STYLE_SELECTED: symbols::border::Set = symbols::border::PROPORTIONAL_TALL;
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).fg(FOCUS_COLOR);
const GAUGE_STYLE: Style = Style::new().fg(LIGHT_COLOR).bg(ALT_ROW_BG_COLOR);

impl App {
    //Renders header
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let text = format!("SerenIT\n{}",if self.get_sound_manager().is_paused() {"[Paused]"} else {""});
        Arc::new(
            Paragraph::new(text)
                .bold()
                .centered()
                .bg(TEAL.c500)
                .fg(YELLOW)
                .render(area, buf),
        );
    }


    //Renders footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let text = if !self.get_mixer_mode() {
            " Tab : switch tab, 's' : save, 'q' : quit\n \
            -/+ : select category, ctrl & -/+ : adjust the master volume\n \
            Enter : add/remove the selected sound, Space : pause/play"
        } else {
            " Tab : switch tab, 's' : save, 'q' : quit\n \
            -/+ : adjust sound volume, ctrl & -/+ : adjust the master volume\n \
            Space : pause/play"
        };
        Paragraph::new(text)
            .left_aligned()
            .bg(FOCUS_UNSELECTED_COLOR)
            .fg(YELLOW)
            .bold()
            .render(area, buf);
    }



    //Renders left list
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        //Category
        let categories = self.get_sound_manager().categories();
        let mut category_text = match self.get_category() {
            Some(i) => format!(
                "{} {}/{}",
                categories[i].to_uppercase(),
                i + 1,
                categories.len()
            ),
            None => "All".to_string(),
        };
        if !self.get_mixer_mode(){
            category_text = format!("{}{}{}","← ",category_text," →");
        }
        let category_line = Line::styled(
            " Category: ".to_string() + &category_text,
            HEADER_STYLE.fg(if self.get_category().is_some() {
                YELLOW
            } else {
                LIGHT_COLOR
            }),
        )
        .left_aligned();

        // Sounds
        let items: Vec<ListItem> = self
            .get_sound_manager()
            .get_sound_list()
            .iter()
            .filter(|s| {
                if let Some(c) = self.get_category() {
                    s.category() == self.get_sound_manager().categories()[c]
                } else {
                    true
                }
            })
            .enumerate()
            .map(|(i, s)| {
                let color = alternate_colors(i);
                let playing = self.get_sound_manager().is_sound_playing(s.path());
                let paused = self.get_sound_manager().is_sound_paused(s.path());

                let mut displayed_name = if self.get_category().is_none() {
                    format!("[{}] {}", s.category().to_uppercase(), s.name())
                } else {
                    s.name().to_string()
                };
                if paused{
                    displayed_name = format!("{} 𝄽", displayed_name);
                } else if playing {
                    displayed_name = format!("{} ♪", displayed_name);
                }

                let mut item = ListItem::from(displayed_name).bg(color);
                if playing {
                    item = item.fg(AMBER.c100);
                }
                item
            })
            .collect();

        //Render
        let border_style = if self.get_mixer_mode() {BORDER_STYLE_NONE} else {BORDER_STYLE_SELECTED};
        let block = Block::new()
            .title(Line::raw("Sounds List").centered())
            .borders(Borders::ALL)
            .border_set(border_style)
            .border_style(HEADER_STYLE)
            .bg(TEAL.c800);

        let [cat_layout, list_layout] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(block.inner(area));
    
        let selected_playing = match self.get_sound_selected_path(){
            Some(path)=>self.get_sound_manager().is_sound_playing(&path),
            None=>false,
        };
        info!("selected_playing: {}",selected_playing);
        let selected_style =
        if self.get_mixer_mode() {
            SELECTED_STYLE.fg(FOCUS_UNSELECTED_COLOR)
        } else if selected_playing {
            SELECTED_STYLE.add_modifier(Modifier::BOLD)
        } else {
            SELECTED_STYLE
        };

        let list = List::new(items)
            .highlight_style(selected_style)
            .highlight_symbol(" =>")
            .highlight_spacing(HighlightSpacing::Always);

        block.render(area, buf);
        Paragraph::new(category_line).render(cat_layout, buf);
        StatefulWidget::render(list, list_layout, buf, &mut self.get_state());
    }





    //Renders right list
    fn render_current_sounds(&self, area: Rect, buf: &mut Buffer) {

        let border_style = if !self.get_mixer_mode() {BORDER_STYLE_NONE} else {BORDER_STYLE_SELECTED};
        let block = Block::new()
            .title(Line::styled("Mixer",HEADER_STYLE).centered())
            .borders(Borders::ALL)
            .border_set(border_style)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let sounds = self.get_sound_manager().playing_sounds();
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
            if !self.get_sound_manager().is_sound_playing(path) {
                return;
            }

            let volume = match self.get_sound_manager().get_sound_by_path(path) {
                Some(sound) => {
                    if self.get_sound_manager().is_sound_paused(path) {
                        0.0
                    } else {
                        sound.volume()
                    }},
                None => 0.0,
            };

            let selected = match self.get_mixer_index() {
                Some(index) => index == i,
                None => false,
            };

            let mut color = if selected {FOCUS_COLOR} else {LIGHT_COLOR};
            let mut gauge_style = if selected {GAUGE_STYLE.fg(FOCUS_COLOR)} else {GAUGE_STYLE};
            if !self.get_mixer_mode() && selected{
                color=FOCUS_UNSELECTED_COLOR;
                gauge_style=gauge_style.fg(FOCUS_UNSELECTED_COLOR);
            }

            Paragraph::new(path)
                .wrap(Wrap { trim: false })
                .fg(color)
                .render(layouts[3 * i], buf);

            Gauge::default()
                .gauge_style(gauge_style)
                .ratio(volume.into())
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

        self.render_header(header_area, buf);
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
