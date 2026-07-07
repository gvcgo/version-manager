mod app;
mod ui;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
