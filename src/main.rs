mod adb;
mod app;
mod cli;
mod error;
mod file_entry;
mod input;
mod local_fs;
mod transfer;
mod ui;

use std::io;

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let _cli = Cli::parse();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let adb = adb::AdbClient::new(None)?;
    let app = App::new(adb, std::env::current_dir()?);

    run_app(&mut terminal, app)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    use crossterm::event::{self, Event as CEvent, KeyEventKind};
    use std::time::Duration;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    input::handle(&mut app, key);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
