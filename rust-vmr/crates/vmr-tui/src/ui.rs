use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

/// Render the TUI frame
pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Title bar
    let title = Paragraph::new("VMR - Version Manager")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    // Main content area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(1),  // Help bar
        ])
        .split(area);

    f.render_widget(title, chunks[0]);

    match app.screen.as_str() {
        "sdk_list" => render_sdk_list(f, chunks[1], app),
        "version_list" => render_version_list(f, chunks[1], app),
        _ => {}
    }

    // Help bar
    let help = match app.screen.as_str() {
        "sdk_list" => " j/k or ↑/↓: navigate  Enter: select  q/Esc: quit ",
        "version_list" => " j/k or ↑/↓: navigate  Enter: install  q/Esc: back ",
        _ => " q/Esc: quit ",
    };
    let help_para = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help_para, chunks[2]);
}

fn render_sdk_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .sdk_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("  {}", name), style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available SDKs"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(list, area);
}

fn render_version_list(f: &mut Frame, area: Rect, app: &App) {
    let header = format!(" Available versions for: {} ", app.current_sdk);

    let items: Vec<ListItem> = app
        .versions
        .iter()
        .enumerate()
        .map(|(i, ver)| {
            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("  {}", ver), style)))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(header))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(list, area);
}
