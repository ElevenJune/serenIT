//use color_eyre::owo_colors::OwoColorize;
use crate::App;
use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{
        palette::tailwind::{AMBER, TEAL},
        Color, Modifier, Style, Stylize,
    },
    symbols::{self},
    text::Line,
    widgets::{
        Block, Borders, Gauge, HighlightSpacing, List, ListItem, Paragraph, StatefulWidget, Tabs,
        Widget, Wrap,
    },
};
use std::{fmt::format, sync::Arc};
use cli_log::*;

const LIGHT_COLOR: Color = TEAL.c100;
const FOCUS_COLOR: Color = AMBER.c300;
const PAUSED_COLOR: Color = AMBER.c500;
const FOCUS_UNSELECTED_COLOR: Color = TEAL.c400;
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
const YELLOW: Color = AMBER.c100;

const HEADER_STYLE: Style = Style::new()
    .fg(LIGHT_COLOR)
    .bg(ALT_ROW_BG_COLOR)
    .add_modifier(Modifier::BOLD);
const BORDER_STYLE_NONE: symbols::border::Set = symbols::border::EMPTY;
const BORDER_STYLE_SELECTED: symbols::border::Set = symbols::border::PROPORTIONAL_TALL;
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).fg(FOCUS_COLOR);
const SELECTED_TAB_STYLE: Style = Style::new().bg(ALT_ROW_BG_COLOR).fg(FOCUS_COLOR);
const NOT_SELECTED_TAB_STYLE: Style = Style::new().bg(ALT_ROW_BG_COLOR).fg(TEAL.c600);
const GAUGE_STYLE: Style = Style::new().fg(LIGHT_COLOR).bg(ALT_ROW_BG_COLOR);

impl App {
    /// Renders header
    fn render_header(&self, area: Rect, buf: &mut Buffer) {
        let mut text = format!("SerenIT\n");
        let mut bg = TEAL.c500;
        if self.get_sound_manager().is_paused() {
            text+="[PAUSED]";
            bg = PAUSED_COLOR;
        }
        /*let text = format!(
            "SerenIT\n{}",
            if self.get_sound_manager().is_paused() {
                "[Paused]"
            } else {
                ""
            }
        );*/
        Arc::new(
            Paragraph::new(text)
                .bold()
                .centered()
                .bg(bg)
                .fg(YELLOW)
                .render(area, buf),
        );
    }

    /// Renders footer
    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let text = if !self.get_mixer_mode() {
            " Tab : switch between sound/scenes, 's' : save, 'q' : quit, 'm' : switch to mixer\n \
            ←→ : select category, ctrl & ←→ : adjust the master volume\n \
            Enter : add/remove the selected sound, Space : pause/play, 'n' : create scene"
        } else {
            " 'm' : go back to calaog, 's' : save, 'q' : quit\n \
            ←→ : adjust sound volume, ctrl & ←→ : adjust the master volume\n \
            Space : pause/play selected sound"
        };
        Paragraph::new(text)
            .left_aligned()
            .bg(FOCUS_UNSELECTED_COLOR)
            .fg(YELLOW)
            .bold()
            .render(area, buf);
    }

    /// Renders tab widget and returns the area for the selected tab
    fn render_sound_scene_tabs(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let border_style = if self.get_mixer_mode() {
            BORDER_STYLE_NONE
        } else {
            BORDER_STYLE_SELECTED
        };
        let block = Block::new()
        .title(Line::styled("Input", HEADER_STYLE).centered())
        .borders(Borders::ALL)
        .border_set(border_style)
        .border_style(HEADER_STYLE)
        .bg(NORMAL_ROW_BG);

        let [_, widget_layout] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(block.inner(area));
        let selected_index = if self.get_sound_list_tab() { 0 } else { 1 };

        Tabs::new(vec!["<Sounds>", "<Scenes>"])
            .block(block)
            .style(NOT_SELECTED_TAB_STYLE)
            .highlight_style(SELECTED_TAB_STYLE)
            .select(selected_index)
            .divider("▌")
            .padding(" ", " ")
            .render(area,buf);

        widget_layout
    }

    /// Renders the sound collection tab
    fn render_sound_collection_tab(&mut self, area: Rect, buf: &mut Buffer) {
        //==Category
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
        if !self.get_mixer_mode() {
            category_text = format!("{}{}{}", "← ", category_text, " →");
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


        //==Sounds
        let items: Vec<ListItem> = self
            .get_sound_manager()
            .get_sound_list()
            .iter()
            //Filter for selected category
            .filter(|s| {
                if let Some(c) = self.get_category() {
                    s.category() == self.get_sound_manager().categories()[c]
                } else {
                    true
                }
            })
            .enumerate()
            //Generate ListItem for each sound
            .map(|(i, s)| {
                let color = alternate_colors(i);
                let playing = self.get_sound_manager().is_sound_playing(s.path());
                let paused = self.get_sound_manager().is_sound_paused(s.path());

                //Display category if no filter
                let category_format = match self.get_category() {
                    None => format!("[{}] {}", s.category().to_uppercase(), s.name()),
                    _ => s.name().to_string()
                };

                //Add playing/paused symbol
                let displayed_name = match (playing, paused) {
                    (_, true) => format!("{} 𝄽", category_format),
                    (true, _) => format!("{} ♪", category_format),
                    _ => category_format,
                };

                //Create ListItem
                ListItem::from(displayed_name)
                .bg(color)
                .fg(match playing{
                    true => YELLOW,
                    false => LIGHT_COLOR
                })
            })
            .collect();

        //Design block
        let block = Block::new()
            .title(Line::raw("Sounds List").centered())
            .borders(Borders::ALL)
            .border_set(BORDER_STYLE_NONE)
            .border_style(HEADER_STYLE)
            .bg(ALT_ROW_BG_COLOR);

        //Split area for category and list
        let [cat_layout, list_layout] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(block.inner(area));


        //Design selected item
        let selected_playing = self.get_sound_selected_path()
        .map_or(false, |path| self.get_sound_manager().is_sound_playing(&path));

        let selected_style = match (self.get_mixer_mode(), selected_playing) {
            (true, _) => SELECTED_STYLE.fg(FOCUS_UNSELECTED_COLOR),
            (_, true) => SELECTED_STYLE.add_modifier(Modifier::BOLD),
            _ => SELECTED_STYLE,
        };

        //Render block, list, and category
        let list = List::new(items)
            .highlight_style(selected_style)
            .highlight_symbol(" =>")
            .highlight_spacing(HighlightSpacing::Always);

        block.render(area, buf);
        Paragraph::new(category_line).render(cat_layout, buf);
        StatefulWidget::render(list, list_layout, buf, &mut self.get_sound_list_state());
    }


    /// Renders the scene collection tab
    fn render_scene_collection_tab(&mut self, area: Rect, buf: &mut Buffer) {
        //==Scenes
        let items: Vec<ListItem> = self
            .get_sound_manager()
            .get_scene_collection()
            .iter()
            .enumerate()
            //Generate ListItem for each sound
            .map(|(i, s)| {
                let name = s.name.to_string();
                let is_playing = i==self.get_sound_manager().get_current_scene_index();

                //Create ListItem
                ListItem::from(name)
                .bg(alternate_colors(i))
                .fg(match is_playing{
                    true => YELLOW,
                    false => LIGHT_COLOR
                })
            })
            .collect();

        //Design block
        let block = Block::new()
            .title(Line::raw("Scene List").centered())
            .borders(Borders::ALL)
            .border_set(BORDER_STYLE_NONE)
            .border_style(HEADER_STYLE)
            .bg(ALT_ROW_BG_COLOR);

        //Design selected item
        let selected_playing = self.get_scene_selected_index()
        .map_or(false, |index| self.get_sound_manager().get_current_scene_index() == index);

        let selected_style = match (self.get_mixer_mode(), selected_playing) {
            (true, _) => SELECTED_STYLE.fg(FOCUS_UNSELECTED_COLOR),
            (_, true) => SELECTED_STYLE.add_modifier(Modifier::BOLD),
            _ => SELECTED_STYLE,
        };

        //Render block, list, and category
        let list = List::new(items)
            .highlight_style(selected_style)
            .highlight_symbol(" =>")
            .highlight_spacing(HighlightSpacing::Always)
            .block(block);
        StatefulWidget::render(list, area, buf, &mut self.get_scene_list_state());
    }


    //Renders the mixer (right panel)
    fn render_mixer(&self, area: Rect, buf: &mut Buffer) {
        let border_style = if !self.get_mixer_mode() {
            BORDER_STYLE_NONE
        } else {
            BORDER_STYLE_SELECTED
        };
        
        let block = Block::new()
            .title(Line::styled("Mixer", HEADER_STYLE).centered())
            .borders(Borders::ALL)
            .border_set(border_style)
            .border_style(HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let sounds = self.get_sound_manager().playing_sounds();
        let max_visible_lines = block.inner(area).height;
        let max_visible_sounds = max_visible_lines/3;
        info!("========================");
        info!("max lines {}, max display {}",max_visible_lines,max_visible_sounds);
        //Each sounds needs 3 lines to be displayed (name, volume bar, space)
        let mut constr: Vec<Constraint> = vec![];
        for _i in 0..sounds.len() {
            if (constr.len()+3)>max_visible_lines.into() {break;}
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
            constr.push(Constraint::Length(1));
        }
        //constr.push(Constraint::Fill(1));

        let layouts = Layout::vertical(constr).split(block.inner(area));

        block.render(area, buf);



        sounds.iter().enumerate().for_each(|(i, (p, _))| {
            let path = p.as_str();
            if !self.get_sound_manager().is_sound_playing(path) {
                return;
            }
            if i>=max_visible_sounds.into() {
                return;
            }
            info!("sound {}",i);
            

            let volume = match self.get_sound_manager().get_sound_by_path(path) {
                Some(sound) => sound.volume(),
                None => 0.0,
            };

            let selected = match self.get_mixer_index() {
                Some(index) => index == i,
                None => false,
            };

            let mut color = if selected { FOCUS_COLOR } else { LIGHT_COLOR };
            let mut gauge_style = if selected {
                GAUGE_STYLE.fg(FOCUS_COLOR)
            } else {
                GAUGE_STYLE
            };
            if !self.get_mixer_mode() && selected {
                color = FOCUS_UNSELECTED_COLOR;
                gauge_style = gauge_style.fg(FOCUS_UNSELECTED_COLOR);
            }
            if self.get_sound_manager().is_paused() {
                gauge_style = GAUGE_STYLE.fg(PAUSED_COLOR);
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

        let [list_area, mixer_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        self.render_header(header_area, buf);
        self.render_footer(footer_area, buf);
        let inner_tab_area = self.render_sound_scene_tabs(list_area, buf);
        self.render_mixer(mixer_area, buf);
        if self.get_sound_list_tab() {
            self.render_sound_collection_tab(inner_tab_area, buf);
        } else {
            self.render_scene_collection_tab(inner_tab_area, buf);
        }
    }
}

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
