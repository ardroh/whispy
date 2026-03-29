use crate::app::UserEvent;
use crate::overlay_html;
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::window::{Window, WindowBuilder};
use wry::{WebView, WebViewBuilder};

pub struct OverlayWindow {
    window: Window,
    webview: WebView,
    visible: bool,
}

impl OverlayWindow {
    /// Create at app startup while no other app has meaningful focus.
    /// The window starts hidden and is shown/hidden via show()/hide().
    pub fn new(event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>) -> Self {
        let (pos_x, pos_y) = screen_bottom_center(event_loop);

        let window = WindowBuilder::new()
            .with_decorations(false)
            .with_always_on_top(true)
            .with_focused(false)
            .with_focusable(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(220.0, 48.0))
            .with_position(LogicalPosition::new(pos_x, pos_y))
            .build(event_loop)
            .expect("Failed to create overlay window");

        configure_ns_window(&window);

        let html = overlay_html::build();
        let webview = WebViewBuilder::new()
            .with_html(&html)
            .with_transparent(true)
            .build(&window)
            .expect("Failed to create overlay webview");

        Self {
            window,
            webview,
            visible: false,
        }
    }

    pub fn show(&mut self) {
        if !self.visible {
            self.visible = true;
            show_without_focus(&self.window);
        }
    }

    pub fn hide(&mut self) {
        if self.visible {
            self.visible = false;
            hide_window(&self.window);
            let _ = self.webview.evaluate_script("reset()");
        }
    }

    pub fn update_levels(&self, levels: &[f32]) {
        let json = format!(
            "[{}]",
            levels
                .iter()
                .map(|l| format!("{:.3}", l))
                .collect::<Vec<_>>()
                .join(",")
        );
        let _ = self
            .webview
            .evaluate_script(&format!("updateLevels({})", json));
    }

    pub fn set_processing(&self) {
        let _ = self.webview.evaluate_script("setProcessing()");
        self.window.request_redraw();
    }
}

fn screen_bottom_center(
    event_loop: &tao::event_loop::EventLoopWindowTarget<UserEvent>,
) -> (f64, f64) {
    let overlay_w = 220.0;
    let overlay_h = 48.0;
    let margin_bottom = 30.0;

    if let Some(monitor) = event_loop.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let screen_w = size.width as f64 / scale;
        let screen_h = size.height as f64 / scale;
        let x = (screen_w - overlay_w) / 2.0;
        let y = screen_h - overlay_h - margin_bottom;
        (x, y)
    } else {
        (600.0, 800.0)
    }
}

#[cfg(target_os = "macos")]
fn configure_ns_window(window: &Window) {
    use objc2_app_kit::{NSColor, NSWindow, NSWindowCollectionBehavior};
    use tao::platform::macos::WindowExtMacOS;

    unsafe {
        let ns_window = window.ns_window() as *mut NSWindow;
        let ns_window = &*ns_window;

        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
        ns_window.setOpaque(false);
        ns_window.setHasShadow(false);
        ns_window.setLevel(25);
        ns_window.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
        ns_window.setIgnoresMouseEvents(true);
    }
}

#[cfg(target_os = "macos")]
fn show_without_focus(window: &Window) {
    use objc2_app_kit::NSWindow;
    use tao::platform::macos::WindowExtMacOS;

    unsafe {
        let ns_window = window.ns_window() as *mut NSWindow;
        (*ns_window).orderFrontRegardless();
    }
}

#[cfg(target_os = "macos")]
fn hide_window(window: &Window) {
    use objc2_app_kit::NSWindow;
    use tao::platform::macos::WindowExtMacOS;

    unsafe {
        let ns_window = window.ns_window() as *mut NSWindow;
        let ns_window = &*ns_window;
        ns_window.orderOut(None);
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_ns_window(_window: &Window) {}
#[cfg(not(target_os = "macos"))]
fn show_without_focus(window: &Window) { window.set_visible(true); }
#[cfg(not(target_os = "macos"))]
fn hide_window(window: &Window) { window.set_visible(false); }
