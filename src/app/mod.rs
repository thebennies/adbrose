use crossterm::event::{KeyCode, KeyEvent};

/// Application state
pub struct App {
    pub title: String,
    pub should_quit: bool,
    pub cursor_index: usize,
    pub items: Vec<String>,
}

impl App {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            should_quit: false,
            cursor_index: 0,
            items: vec![
                "Item 1".into(),
                "Item 2".into(),
                "Item 3".into(),
            ],
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor_index = (self.cursor_index + 1).min(self.items.len() - 1);
                false
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor_index = self.cursor_index.saturating_sub(1);
                false
            }
            _ => false,
        }
    }

    pub fn tick(&self) {}
}
