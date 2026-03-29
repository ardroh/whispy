//! macOS privacy prompts (microphone + accessibility).

use crate::app::show_notification;
#[cfg(target_os = "macos")]
use crate::audio::AudioRecorder;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::boolean::CFBoolean;
#[cfg(target_os = "macos")]
use core_foundation::dictionary::CFDictionary;
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> u8;
}

fn support_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .context("Could not find Application Support directory")?
        .join("whispy");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn first_launch_marker_path() -> Result<PathBuf> {
    Ok(support_dir()?.join(".permissions_intro_shown"))
}

/// On first run, triggers the accessibility trust dialog and microphone access probe.
pub fn ensure_prompted_on_first_launch() {
    let Ok(path) = first_launch_marker_path() else {
        return;
    };
    if path.exists() {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        prompt_accessibility_if_needed();
        if let Ok(mut recorder) = AudioRecorder::new() {
            if let Err(e) = recorder.request_microphone_access() {
                tracing::warn!("Microphone permission probe: {}", e);
            }
        }
    }

    let _ = fs::write(path, "");
}

/// Menu action: re-run probes and summarize status in a notification.
pub fn check_permissions_interactive() {
    #[cfg(target_os = "macos")]
    {
        let (mic_ok, mic_line): (bool, String) =
            match AudioRecorder::new().and_then(|mut r| r.request_microphone_access()) {
                Ok(()) => (
                    true,
                    "Microphone: OK (or system prompt was shown).".to_string(),
                ),
                Err(e) => (
                    false,
                    format!(
                        "Microphone: {} — enable Whispy under Privacy & Security → Microphone.",
                        e
                    ),
                ),
            };

        let ax_before = accessibility_trusted();
        if !ax_before {
            prompt_accessibility_if_needed();
        }
        let ax_ok = accessibility_trusted();
        let ax_line: String = if ax_ok {
            "Accessibility: granted.".to_string()
        } else {
            "Accessibility: not granted — enable Whispy under Privacy & Security → Accessibility."
                .to_string()
        };

        show_notification(
            "Whispy — permissions",
            &format!("{}\n{}", mic_line, ax_line),
        );

        if !mic_ok {
            let _ = open_microphone_settings();
        }
        if !ax_ok {
            let _ = open_accessibility_settings();
        }
    }
}

#[cfg(target_os = "macos")]
fn accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) != 0 }
}

#[cfg(target_os = "macos")]
fn prompt_accessibility_if_needed() {
    if accessibility_trusted() {
        return;
    }
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let dict = CFDictionary::from_CFType_pairs(&[(key, value)]);
    unsafe {
        let _ = AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef());
    }
}

#[cfg(target_os = "macos")]
fn open_microphone_settings() -> std::process::ExitStatus {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        .status()
        .unwrap_or_else(|_| std::process::Command::new("true").status().unwrap())
}

#[cfg(target_os = "macos")]
fn open_accessibility_settings() -> std::process::ExitStatus {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .status()
        .unwrap_or_else(|_| std::process::Command::new("true").status().unwrap())
}
