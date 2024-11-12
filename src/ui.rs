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
use std::sync::Arc;
use crate::App;

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
const YELLOW: Color = AMBER.c100;
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).add_modifier(Modifier::BOLD);
//const EDIT_VALUE_COLOR: Color = AMBER.c500;
/*const EDIT_STYLE: Style = Style::new()
    .bg(EDIT_ROW_COLOR)
    .add_modifier(Modifier::BOLD)
    .fg(AMBER.c100);
const EDIT_VALUE_STYLE: Style = Style::new()
    .bg(EDIT_VALUE_COLOR)
    .add_modifier(Modifier::BOLD)
    .fg(AMBER.c100);
const TEXT_FG_COLOR: Color = TEAL.c200;
const TEXT_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);*/

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
        let text = "Add/Remove the selected sound with Enter, pause/play with space\n\
            -/+ to adjust the volume, ctrl & -/+ to adjust the master volume\n\
            's' to save, 'c' to swicth category, 'q' to quit";
        Paragraph::new(text)
            .centered()
            .bg(TEAL.c500)
            .fg(YELLOW)
            .bold()
            .render(area, buf);
    }

    //Renders left list
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        //Category
        let categories = self.get_sound_manager().categories();
        let category_text = match self.get_category() {
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
            TODO_HEADER_STYLE.fg(if self.get_category().is_some() {
                YELLOW
            } else {
                TEAL.c100
            }),
        )
        .centered();

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
                let displayed_name = if self.get_category().is_none() {
                    format!("[{}] {}", s.category().to_uppercase(), s.name())
                } else {
                    s.name().to_string()
                };
                let mut item = ListItem::from(displayed_name).bg(color);
                if self.get_sound_manager().is_sound_playing(s.path()) {
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
        StatefulWidget::render(list, list_layout, buf, &mut self.get_state());
    }

    fn render_current_sounds(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::styled("Mixer",TODO_HEADER_STYLE).centered())
            .borders(Borders::LEFT)
            .border_set(symbols::border::PROPORTIONAL_TALL)
            .border_style(MIXER_BORDERS_STYLE)
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
