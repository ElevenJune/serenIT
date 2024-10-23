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
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    }
};
use crate::App;


const TODO_HEADER_STYLE: Style = Style::new().fg(TEAL.c100).bg(TEAL.c800);
const NORMAL_ROW_BG: Color = TEAL.c900;
const ALT_ROW_BG_COLOR: Color = TEAL.c800;
const EDIT_ROW_COLOR: Color = AMBER.c700;
const EDIT_VALUE_COLOR: Color = AMBER.c500;
const EDIT_STYLE: Style = Style::new().bg(EDIT_ROW_COLOR).add_modifier(Modifier::BOLD).fg(AMBER.c100);
const EDIT_VALUE_STYLE: Style = Style::new().bg(EDIT_VALUE_COLOR).add_modifier(Modifier::BOLD).fg(AMBER.c100);
const SELECTED_STYLE: Style = Style::new().bg(TEAL.c600).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = TEAL.c200;
const TEXT_STYLE: Style = Style::new().fg(TEXT_FG_COLOR);

impl App {

    //Renders header
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Todo List Application")
            .bold()
            .centered()
            .bg(TEAL.c500)
            .render(area, buf);
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
    /*fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Task List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(TODO_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .get_list()
            .items()
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                let displayed_name = todo_item.name.clone();
                let mut item = ListItem::from(displayed_name).bg(color);
                if todo_item.done {
                    item = item.add_modifier(Modifier::CROSSED_OUT);
                }
                else if todo_item.priority>5 {
                    item = item.add_modifier(Modifier::BOLD).fg(AMBER.c100);
                }
                item
            })
            .collect();

        let mut selected_style = SELECTED_STYLE;
        let mut symbol = " => ";
        if self.is_edit_mode() {
            symbol = "===>";
            selected_style=EDIT_STYLE;//.add_modifier(Modifier::REVERSED);
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(selected_style)
            .highlight_symbol(symbol)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.get_state());
    }

    //Renders selected task (right)
    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
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
        let footer_length = if true {3} else {2};
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(footer_length),
        ])
        .areas(area);

        let info_weight = if true {2} else {1};
        let [list_area, item_area] =
            Layout::horizontal([Constraint::Fill(3-info_weight), Constraint::Fill(info_weight)]).areas(main_area);

        App::render_header(header_area, buf);
        self.render_footer(footer_area, buf);
    }
}

pub const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}