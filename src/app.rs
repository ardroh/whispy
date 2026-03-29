use crate::audio::{raw_to_wav, AudioRecorder};
use crate::config::Config;
use crate::paste;
use crate::transcribe;
use crate::tray::Tray;
use anyhow::Result;
use tao::event_loop::EventLoopProxy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Idle,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    TranscriptionComplete(Result<String, String>),
}

pub struct AppState {
    pub phase: Phase,
    pub config: Config,
    recorder: AudioRecorder,
    http_client: reqwest::Client,
    event_proxy: EventLoopProxy<UserEvent>,
}

impl AppState {
    pub fn new(event_proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        let recorder = AudioRecorder::new()?;
        let http_client = reqwest::Client::new();
        Ok(Self {
            phase: Phase::Idle,
            config,
            recorder,
            http_client,
            event_proxy,
        })
    }

    pub fn audio_levels(&self, num_bands: usize) -> Vec<f32> {
        self.recorder.audio_levels(num_bands)
    }

    pub fn reload_config(&mut self) {
        if let Ok(config) = Config::load() {
            self.config = config;
            tracing::info!("Config reloaded");
        }
    }

    pub fn toggle_recording(&mut self, tray: &Tray) {
        match self.phase {
            Phase::Idle => self.start_recording(tray),
            Phase::Recording => self.stop_recording(tray),
            Phase::Transcribing => {
                tracing::warn!("Already transcribing, ignoring hotkey");
            }
        }
    }

    fn start_recording(&mut self, tray: &Tray) {
        // Reload config in case it was changed in preferences
        self.reload_config();

        if !self.config.has_api_key() {
            tracing::error!("No API key configured");
            show_notification("Whispy", "Please set your OpenAI API key in Preferences.");
            return;
        }

        match self.recorder.start() {
            Ok(()) => {
                self.phase = Phase::Recording;
                tray.set_recording(true);
                tracing::info!("Recording started");
            }
            Err(e) => {
                tracing::error!("Failed to start recording: {}", e);
                show_notification("Whispy", &format!("Failed to start recording: {}", e));
            }
        }
    }

    fn stop_recording(&mut self, tray: &Tray) {
        self.phase = Phase::Transcribing;
        tray.set_transcribing();

        let raw = match self.recorder.end_capture() {
            Ok(r) => r,
            Err(e) => {
                self.phase = Phase::Idle;
                tray.set_idle();
                tracing::error!("Failed to stop recording: {}", e);
                let msg = e.to_string();
                let body = if msg.contains("No audio recorded") {
                    "No audio recorded.".to_string()
                } else {
                    format!("Recording error: {}", msg)
                };
                show_notification("Whispy", &body);
                return;
            }
        };

        let client = self.http_client.clone();
        let api_key = self.config.api_key.clone().unwrap_or_default();
        let model = self.config.model.clone();
        let language = self.config.language.clone();
        let proxy = self.event_proxy.clone();

        std::thread::spawn(move || {
            let wav_data = match raw_to_wav(raw) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Failed to encode WAV: {}", e);
                    let _ = proxy.send_event(UserEvent::TranscriptionComplete(Err(
                        format!("Recording error: {}", e),
                    )));
                    return;
                }
            };

            tracing::info!("Recording stopped, {} bytes of WAV data", wav_data.len());

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let result = rt.block_on(transcribe::transcribe(
                &client, &api_key, &model, &language, wav_data,
            ));

            let event = match result {
                Ok(text) => UserEvent::TranscriptionComplete(Ok(text)),
                Err(e) => UserEvent::TranscriptionComplete(Err(format!(
                    "Transcription failed: {}",
                    e
                ))),
            };
            let _ = proxy.send_event(event);
        });
    }

    pub fn handle_transcription(&mut self, tray: &Tray, result: Result<String, String>) {
        self.phase = Phase::Idle;
        tray.set_idle();

        match result {
            Ok(text) => {
                if text.is_empty() {
                    tracing::warn!("Empty transcription received");
                    show_notification("Whispy", "No speech detected.");
                    return;
                }

                tracing::info!("Transcription: {}", text);
                if let Err(e) = paste::paste_text(&text) {
                    tracing::error!("Failed to paste text: {}", e);
                    show_notification("Whispy", &format!("Failed to paste: {}", e));
                }
            }
            Err(e) => {
                tracing::error!("{}", e);
                show_notification("Whispy", &e);
            }
        }
    }
}

pub(crate) fn show_notification(title: &str, message: &str) {
    let script = format!(
        r#"display notification "{}" with title "{}""#,
        message.replace('"', r#"\""#),
        title.replace('"', r#"\""#)
    );
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .spawn();
}
