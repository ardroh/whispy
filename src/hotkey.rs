use anyhow::Result;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

pub struct HotkeyHandler {
    manager: GlobalHotKeyManager,
    current_hotkey: HotKey,
}

impl HotkeyHandler {
    pub fn new(hotkey_str: &str) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let hotkey = parse_hotkey(hotkey_str);
        manager.register(hotkey)?;
        tracing::info!("Global hotkey registered: {}", hotkey_str);

        Ok(Self {
            manager,
            current_hotkey: hotkey,
        })
    }

    pub fn update_hotkey(&mut self, hotkey_str: &str) -> Result<()> {
        let new_hotkey = parse_hotkey(hotkey_str);
        if new_hotkey.id() == self.current_hotkey.id() {
            return Ok(());
        }
        let _ = self.manager.unregister(self.current_hotkey);
        self.manager.register(new_hotkey)?;
        self.current_hotkey = new_hotkey;
        tracing::info!("Global hotkey updated: {}", hotkey_str);
        Ok(())
    }

    /// Drain pending hotkey events. Returns true if our shortcut was pressed at least once.
    /// (Global hotkeys wake the event loop with `WaitCancelled`; we must not only poll on timer ticks.)
    pub fn poll_hotkey_pressed(&self) -> bool {
        let mut hit = false;
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id() == self.current_hotkey.id()
                && event.state() == global_hotkey::HotKeyState::Pressed
            {
                hit = true;
            }
        }
        hit
    }
}

fn parse_hotkey(s: &str) -> HotKey {
    match s {
        "cmd+shift+space" => HotKey::new(
            Some(Modifiers::SHIFT | Modifiers::META),
            Code::Space,
        ),
        _ => HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::META),
            Code::Space,
        ),
    }
}
