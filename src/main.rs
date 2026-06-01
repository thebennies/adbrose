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
use std::panic;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tracing::metadata::LevelFilter;

use app::{App, Modal};
use cli::Cli;
use file_entry::Device;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref log_path) = cli.log_file {
        let file = std::fs::File::create(log_path)?;
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .with_writer(file)
            .with_ansi(false)
            .init();
    }

    let local_path = cli
        .local_path
        .as_ref()
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    let android_path = cli
        .android_path
        .clone()
        .unwrap_or_else(|| "/sdcard".to_string());

    let serial = cli.serial.clone();
    let adb = match adb::AdbClient::new(serial) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    set_panic_hook();

    let mut app = App::new(adb, local_path);
    app.android.path = std::path::PathBuf::from(android_path);

    if app.adb.serial.is_none() {
        match app.adb.device_list() {
            Ok(devices) => {
                let connected: Vec<Device> = devices
                    .into_iter()
                    .filter(|d| d.state == "device")
                    .collect();
                match connected.len() {
                    0 => {
                        app.modal = Some(Modal::Error {
                            message: "No Android device connected. Connect a device and press R to retry.".into(),
                        });
                    }
                    1 => {
                        app.adb.serial = Some(connected[0].serial.clone());
                    }
                    _ => {
                        app.modal = Some(Modal::DevicePicker { devices: connected });
                    }
                }
            }
            Err(e) => {
                app.modal = Some(Modal::Error {
                    message: format!("Failed to detect devices: {}", e),
                });
            }
        }
    }

    if app.adb.serial.is_some() {
        if let Err(e) = load_initial_dirs(&mut app) {
            app.show_error(format!("Failed to load directories: {}", e));
        }
    } else {
        let _ = load_local_dir(&mut app);
    }

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result?;

    Ok(())
}

fn load_initial_dirs(app: &mut App) -> Result<()> {
    load_local_dir(app)?;
    load_android_dir(app)?;
    Ok(())
}

fn load_local_dir(app: &mut App) -> Result<()> {
    match local_fs::list_dir(&app.local.path) {
        Ok(entries) => {
            app.local.set_entries(entries);
            Ok(())
        }
        Err(e) => {
            app.local.loading = false;
            app.show_error(format!("Failed to list local dir: {}", e));
            Err(anyhow::anyhow!("{}", e))
        }
    }
}

fn load_android_dir(app: &mut App) -> Result<()> {
    match app.adb.list_dir(app.android.path.to_str().unwrap_or("/sdcard")) {
        Ok(entries) => {
            app.android.set_entries(entries);
            Ok(())
        }
        Err(e) => {
            app.android.loading = false;
            app.show_error(format!("Failed to list Android dir: {}", e));
            Err(anyhow::anyhow!("{}", e))
        }
    }
}

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        let _ = execute!(io::stdout(), crossterm::cursor::Show);
        eprintln!("panic: {}", panic_info);
    }));
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            if let CEvent::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    input::handle(app, key);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
