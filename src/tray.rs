use anyhow::Result;
use image::ImageFormat;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const IDLE_PNG: &[u8] = include_bytes!("../assets/tray-idle.png");
const RECORDING_PNG: &[u8] = include_bytes!("../assets/tray-recording.png");

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
        let idle_icon = icon_from_png(IDLE_PNG)?;
        let recording_icon = icon_from_png(RECORDING_PNG)?;

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

        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Whispy - Voice to Text")
            .with_icon(idle_icon.clone());
        #[cfg(target_os = "macos")]
        {
            builder = builder.with_icon_as_template(true);
        }
        let tray = builder.build()?;

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
        #[cfg(target_os = "macos")]
        {
            let _ = self
                .tray
                .set_icon_with_as_template(Some(self.idle_icon.clone()), true);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = self.tray.set_icon(Some(self.idle_icon.clone()));
        }
        let _ = self.tray.set_tooltip(Some("Whispy - Voice to Text"));
    }

    pub fn set_recording(&self, recording: bool) {
        if recording {
            #[cfg(target_os = "macos")]
            {
                let _ = self.tray.set_icon_with_as_template(
                    Some(self.recording_icon.clone()),
                    false,
                );
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = self.tray.set_icon(Some(self.recording_icon.clone()));
            }
            let _ = self.tray.set_tooltip(Some("Whispy - Recording..."));
        } else {
            self.set_idle();
        }
    }

    pub fn set_transcribing(&self) {
        #[cfg(target_os = "macos")]
        {
            let _ = self
                .tray
                .set_icon_with_as_template(Some(self.idle_icon.clone()), true);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = self.tray.set_icon(Some(self.idle_icon.clone()));
        }
        let _ = self.tray.set_tooltip(Some("Whispy - Transcribing..."));
    }

    pub fn check_menu_event(&self) -> Option<MenuId> {
        MenuEvent::receiver().try_recv().ok().map(|e| e.id().clone())
    }
}

fn icon_from_png(bytes: &[u8]) -> Result<Icon> {
    let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)?.into_rgba8();
    let (w, h) = img.dimensions();
    Ok(Icon::from_rgba(img.into_raw(), w, h)?)
}
