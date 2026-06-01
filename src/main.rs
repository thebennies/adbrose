mod app;
mod event;
mod ui;

use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use event::{Event, EventHandler};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App setup
    let events = EventHandler::new(250);
    let mut app = App::new("adbrowse");

    // Main loop
    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        match events.next()? {
            Event::Key(key_event) => {
                if app.handle_key_event(key_event) {
                    break;
                }
            }
            Event::Resize(_, _) => {}
            Event::Tick => {}
        }
    }

    // Terminal restore
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
