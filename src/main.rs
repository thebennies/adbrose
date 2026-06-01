mod adb;
mod app;
mod cli;
mod config;
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
use tokio::sync::mpsc;
use tracing::metadata::LevelFilter;

use app::{App, Modal};
use cli::Cli;
use file_entry::Device;

enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Tick,
    TransferProgress { id: usize, percent: u8 },
    TransferDone { id: usize, result: std::result::Result<(), String> },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(ref log_path) = cli.log_file {
        let file = std::fs::File::create(log_path)?;
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .with_writer(file)
            .with_ansi(false)
            .init();
    }

    let cfg = config::load();

    let local_path = cli
        .local_path
        .or(cfg.defaults.local_path.clone())
        .map(|p| std::path::PathBuf::from(&p))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    let android_path = cli
        .android_path
        .or(cfg.defaults.android_path.clone())
        .unwrap_or_else(|| "/sdcard".to_string());

    let serial = cli.serial.or(cfg.defaults.serial.clone());
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

    let mut app = App::new(adb, local_path, cfg);
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

    let (tx, mut rx) = mpsc::channel(100);

    let event_tx = tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(CEvent::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        if event_tx.blocking_send(AppEvent::Key(key)).is_err() {
                            break;
                        }
                    }
                }
            }
            if event_tx.blocking_send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    let result = run_app(&mut terminal, &mut app, &mut rx, tx).await;

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

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    rx: &mut mpsc::Receiver<AppEvent>,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Key(key) => {
                    input::handle(app, key);
                    spawn_next_transfer(app, &tx);
                }
                AppEvent::Tick => {}
                AppEvent::TransferProgress { id, percent } => {
                    app.transfer_queue.update_progress(id, percent);
                }
                AppEvent::TransferDone { id, result } => {
                    match result {
                        Ok(()) => {
                            app.transfer_queue.mark_completed(id);
                            let _ = app.refresh_active_pane();
                        }
                        Err(e) => {
                            app.transfer_queue.mark_failed(id, e);
                        }
                    }
                    spawn_next_transfer(app, &tx);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn spawn_next_transfer(app: &mut App, tx: &mpsc::Sender<AppEvent>) {
    if app.transfer_queue.has_active() {
        return;
    }
    if let Some(job) = app.transfer_queue.next_pending() {
        let id = job.id;
        let src = job.source.clone();
        let dst = job.destination.clone();
        let to_android = job.to_android;
        let is_dir = job.is_dir;
        let adb = app.adb.clone();
        let use_tar = app.config.transfer.use_tar_streaming;
        let tx = tx.clone();
        app.transfer_queue.mark_started(id);

        tokio::spawn(async move {
            let src_str = src.to_str().unwrap_or("").to_string();
            let dst_str = dst.to_str().unwrap_or("").to_string();
            let dst_parent = dst.parent()
                .map(|p| p.to_str().unwrap_or(".").to_string())
                .unwrap_or_else(|| ".".to_string());

            let tx_done = tx.clone();
            let result = if to_android {
                let tx_p = tx.clone();
                adb.push_with_progress(&src_str, &dst_str, move |p| {
                    let _ = tx_p.blocking_send(AppEvent::TransferProgress { id, percent: p });
                }).await
            } else if is_dir && use_tar {
                adb.tar_pull(&src_str, &dst_parent).await
            } else {
                let tx_p = tx.clone();
                adb.pull_with_progress(&src_str, &dst_str, move |p| {
                    let _ = tx_p.blocking_send(AppEvent::TransferProgress { id, percent: p });
                }).await
            };

            let msg = match result {
                Ok(()) => Ok(()),
                Err(e) => Err(e.to_string()),
            };
            let _ = tx_done.send(AppEvent::TransferDone { id, result: msg }).await;
        });
    }
}
