mod app;
mod audio;
mod config;
mod hotkey;
mod overlay;
mod overlay_html;
mod paste;
mod permissions;
mod preferences;
mod preferences_html;
mod transcribe;
mod tray;

use app::{AppState, UserEvent};
use hotkey::HotkeyHandler;
use preferences::PreferencesWindow;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use std::time::{Duration, Instant};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn log_path() -> std::path::PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("whispy");
    std::fs::create_dir_all(&dir).ok();
    dir.join("whispy.log")
}

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .expect("Failed to open log file");

    let (file_writer, guard) = tracing_appender::non_blocking(log_file);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false);

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(file_layer)
        .with(stderr_layer)
        .init();

    guard
}

fn main() {
    let _log_guard = init_logging();
    tracing::info!("Whispy starting...");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    let mut state = AppState::new(event_loop.create_proxy())
        .expect("Failed to initialize app state");

    permissions::ensure_prompted_on_first_launch();

    let tray = tray::Tray::new().expect("Failed to create tray icon");
    let mut hotkey_handler = HotkeyHandler::new(&state.config.hotkey)
        .expect("Failed to register global hotkey");
    let mut prefs_window: Option<PreferencesWindow> = None;

    let mut overlay: Option<overlay::OverlayWindow> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16));

        match event {
            Event::NewEvents(StartCause::Init) => {
                tracing::info!("Whispy ready");
                // Create overlay at startup (hidden) so showing it later
                // never steals focus from the active app
                overlay = Some(overlay::OverlayWindow::new(event_loop));
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
                ..
            } => {
                if prefs_window
                    .as_ref()
                    .is_some_and(|pw| pw.window_id == window_id)
                {
                    prefs_window = None;
                    state.reload_config();
                    if let Err(e) = hotkey_handler.update_hotkey(&state.config.hotkey) {
                        tracing::error!("Failed to update hotkey: {}", e);
                    }
                }
            }

            Event::UserEvent(UserEvent::TranscriptionComplete(result)) => {
                state.handle_transcription(&tray, result);
            }

            // Run once per event-loop tick, including when a global hotkey wakes the loop
            // (`WaitCancelled`). Hotkeys were previously only handled on `ResumeTimeReached`,
            // which could delay stop/start by a full timer interval or more.
            Event::MainEventsCleared => {
                if hotkey_handler.poll_hotkey_pressed() {
                    if state.phase == app::Phase::Recording {
                        if let Some(ref ov) = overlay {
                            ov.set_processing();
                        }
                    }
                    state.toggle_recording(&tray);
                }

                if let Some(menu_id) = tray.check_menu_event() {
                    if menu_id == tray.quit_id {
                        *control_flow = ControlFlow::Exit;
                    } else if menu_id == tray.prefs_id {
                        if prefs_window.is_none() {
                            prefs_window =
                                Some(PreferencesWindow::new(event_loop, &state.config));
                        }
                    } else if menu_id == tray.logs_id {
                        open_log_file();
                    } else if menu_id == tray.perms_id {
                        permissions::check_permissions_interactive();
                    }
                }

                if let Some(ref mut ov) = overlay {
                    match state.phase {
                        app::Phase::Recording => {
                            ov.show();
                            ov.update_levels(&state.audio_levels(20));
                        }
                        app::Phase::Transcribing => {}
                        app::Phase::Idle => {
                            ov.hide();
                        }
                    }
                }
            }

            _ => {}
        }
    });
}

fn open_log_file() {
    let path = log_path();
    if path.exists() {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Console")
            .arg(&path)
            .spawn();
    }
}
