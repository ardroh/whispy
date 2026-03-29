use anyhow::Result;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct Tray {
    tray: TrayIcon,
    idle_icon: Icon,
    recording_icon: Icon,
    pub quit_id: MenuId,
    pub prefs_id: MenuId,
    pub logs_id: MenuId,
    pub perms_id: MenuId,
}

impl Tray {
    pub fn new() -> Result<Self> {
        let idle_icon = create_idle_icon()?;
        let recording_icon = create_recording_icon()?;

        let quit_item = MenuItem::new("Quit", true, None);
        let prefs_item = MenuItem::new("Preferences...", true, None);
        let logs_item = MenuItem::new("View Logs...", true, None);
        let perms_item = MenuItem::new("Check Permissions...", true, None);

        let quit_id = quit_item.id().clone();
        let prefs_id = prefs_item.id().clone();
        let logs_id = logs_item.id().clone();
        let perms_id = perms_item.id().clone();

        let menu = Menu::new();
        let _ = menu.append(&MenuItem::new("Whispy", false, None));
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&prefs_item);
        let _ = menu.append(&logs_item);
        let _ = menu.append(&perms_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Whispy - Voice to Text")
            .with_icon(idle_icon.clone())
            .build()?;

        Ok(Self {
            tray,
            idle_icon,
            recording_icon,
            quit_id,
            prefs_id,
            logs_id,
            perms_id,
        })
    }

    pub fn set_idle(&self) {
        let _ = self.tray.set_icon(Some(self.idle_icon.clone()));
        let _ = self.tray.set_tooltip(Some("Whispy - Voice to Text"));
    }

    pub fn set_recording(&self, recording: bool) {
        if recording {
            let _ = self.tray.set_icon(Some(self.recording_icon.clone()));
            let _ = self.tray.set_tooltip(Some("Whispy - Recording..."));
        } else {
            self.set_idle();
        }
    }

    pub fn set_transcribing(&self) {
        let _ = self.tray.set_icon(Some(self.idle_icon.clone()));
        let _ = self.tray.set_tooltip(Some("Whispy - Transcribing..."));
    }

    pub fn check_menu_event(&self) -> Option<MenuId> {
        MenuEvent::receiver().try_recv().ok().map(|e| e.id().clone())
    }
}

fn create_idle_icon() -> Result<Icon> {
    // 22x22 microphone icon (simple dark circle with mic shape)
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;

            // Microphone body (rounded rect in center)
            let in_mic_body = dx.abs() <= 3.0 && dy >= -6.0 && dy <= 2.0;
            // Mic top (rounded)
            let in_mic_top = dx * dx + (dy + 6.0) * (dy + 6.0) <= 9.0 && dy <= -3.0;
            // Stand arc
            let dist = (dx * dx + (dy - 2.0) * (dy - 2.0)).sqrt();
            let in_arc = dist >= 4.5 && dist <= 6.0 && dy >= 2.0 && dy <= 6.0;
            // Stand line
            let in_stand = dx.abs() <= 1.0 && dy >= 6.0 && dy <= 9.0;
            // Stand base
            let in_base = dx.abs() <= 3.0 && dy >= 8.5 && dy <= 10.0;

            if in_mic_body || in_mic_top || in_arc || in_stand || in_base {
                rgba[idx] = 40;     // R
                rgba[idx + 1] = 40; // G
                rgba[idx + 2] = 40; // B
                rgba[idx + 3] = 220; // A
            }
        }
    }

    Ok(Icon::from_rgba(rgba, size, size)?)
}

fn create_recording_icon() -> Result<Icon> {
    // 22x22 red recording dot
    let size = 22u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let radius = 8.0f32;

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                let alpha = if dist > radius - 1.0 {
                    ((radius - dist) * 255.0) as u8
                } else {
                    255
                };
                rgba[idx] = 220;     // R
                rgba[idx + 1] = 50;  // G
                rgba[idx + 2] = 50;  // B
                rgba[idx + 3] = alpha;
            }
        }
    }

    Ok(Icon::from_rgba(rgba, size, size)?)
}
