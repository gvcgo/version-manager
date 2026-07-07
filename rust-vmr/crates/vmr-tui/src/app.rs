use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use crate::ui;

/// TUI application state.
pub struct App {
    /// Currently selected index in the active list.
    pub selected: usize,
    /// The active screen: "sdk_list", "version_list", "installed_list"
    pub screen: String,
    /// List of SDK names (for sdk_list screen)
    pub sdk_names: Vec<String>,
    /// List of version strings (for version_list screen)
    pub versions: Vec<String>,
    /// Currently selected SDK name
    pub current_sdk: String,
    /// Should exit the application
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            selected: 0,
            screen: "sdk_list".to_string(),
            sdk_names: vec![
                "golang".into(),
                "python".into(),
                "node".into(),
                "rust".into(),
                "java".into(),
                "zig".into(),
                "bun".into(),
                "deno".into(),
                "php".into(),
            ],
            versions: vec![],
            current_sdk: String::new(),
            should_quit: false,
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        let len = self.current_list_len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
        }
    }

    fn current_list_len(&self) -> usize {
        match self.screen.as_str() {
            "sdk_list" => self.sdk_names.len(),
            "version_list" => self.versions.len(),
            _ => 0,
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.screen == "version_list" {
                    // Go back to SDK list
                    self.screen = "sdk_list".to_string();
                    self.selected = 0;
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Enter => {
                if self.screen == "sdk_list" {
                    // Select an SDK — show versions (placeholder)
                    if let Some(sdk) = self.sdk_names.get(self.selected) {
                        self.current_sdk = sdk.clone();
                        self.screen = "version_list".to_string();
                        self.versions = vec![
                            "1.0.0".into(),
                            "1.1.0".into(),
                            "1.2.0".into(),
                            "2.0.0".into(),
                            "2.1.0".into(),
                            "latest".into(),
                        ];
                        self.selected = self.versions.len() - 1; // Select latest
                    }
                }
            }
            _ => {}
        }
    }

    /// Main event loop
    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| ui::render(f, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key.code);
                }
            }
        }
        Ok(())
    }
}
