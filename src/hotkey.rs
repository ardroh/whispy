use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

pub struct HotkeyHandler {
    manager: GlobalHotKeyManager,
    record_hotkey: HotKey,
    pause_hotkey: HotKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleRecording,
    TogglePause,
}

impl HotkeyHandler {
    pub fn new(hotkey_str: &str) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let record_hotkey = parse_hotkey(hotkey_str);
        let pause_hotkey = parse_pause_hotkey();
        manager.register(record_hotkey)?;
        if let Err(e) = manager.register(pause_hotkey) {
            let _ = manager.unregister(record_hotkey);
            return Err(e.into());
        }
        tracing::info!(
            "Global hotkeys registered: {} to start/finish, {} to pause/resume",
            hotkey_str,
            pause_hotkey_label()
        );

        Ok(Self {
            manager,
            record_hotkey,
            pause_hotkey,
        })
    }

    pub fn update_hotkey(&mut self, hotkey_str: &str) -> Result<()> {
        let new_record = parse_hotkey(hotkey_str);
        let new_pause = parse_pause_hotkey();
        if new_record.id() == self.record_hotkey.id() && new_pause.id() == self.pause_hotkey.id() {
            return Ok(());
        }

        // Register the complete replacement pair before removing the current
        // pair so a conflict never leaves the app without a working shortcut.
        self.manager.register(new_record)?;
        if let Err(e) = self.manager.register(new_pause) {
            let _ = self.manager.unregister(new_record);
            return Err(e.into());
        }

        let _ = self.manager.unregister(self.record_hotkey);
        let _ = self.manager.unregister(self.pause_hotkey);
        self.record_hotkey = new_record;
        self.pause_hotkey = new_pause;
        tracing::info!(
            "Global hotkeys updated: {} to start/finish, {} to pause/resume",
            hotkey_str,
            pause_hotkey_label()
        );
        Ok(())
    }

    /// Drain pending hotkey events and return the first relevant action.
    /// (Global hotkeys wake the event loop with `WaitCancelled`; we must not only poll on timer ticks.)
    pub fn poll_action(&self) -> Option<HotkeyAction> {
        let mut action = None;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state() != global_hotkey::HotKeyState::Pressed {
                continue;
            }
            if event.id() == self.record_hotkey.id() {
                action.get_or_insert(HotkeyAction::ToggleRecording);
            } else if event.id() == self.pause_hotkey.id() {
                action = Some(HotkeyAction::TogglePause);
            }
        }
        action
    }
}

fn parse_hotkey(s: &str) -> HotKey {
    match s {
        "cmd+shift+space" => HotKey::new(Some(Modifiers::SHIFT | Modifiers::META), Code::Space),
        _ => HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::META),
            Code::Space,
        ),
    }
}

fn parse_pause_hotkey() -> HotKey {
    HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP)
}

pub fn pause_hotkey_label() -> &'static str {
    "Ctrl + Shift + P"
}

pub fn record_hotkey_label(s: &str) -> &'static str {
    match s {
        "cmd+shift+space" => "Cmd + Shift + Space",
        _ => "Ctrl + Shift + Cmd + Space",
    }
}

#[cfg(test)]
mod tests {
    use super::{pause_hotkey_label, record_hotkey_label};

    #[test]
    fn paired_shortcut_labels_follow_record_shortcut() {
        assert_eq!(
            record_hotkey_label("ctrl+shift+cmd+space"),
            "Ctrl + Shift + Cmd + Space"
        );
        assert_eq!(pause_hotkey_label(), "Ctrl + Shift + P");
        assert_eq!(
            record_hotkey_label("cmd+shift+space"),
            "Cmd + Shift + Space"
        );
    }
}
