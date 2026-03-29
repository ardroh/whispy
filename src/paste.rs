use anyhow::Result;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub fn paste_text(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()?;

    // Save current clipboard contents
    let previous = clipboard.get_text().ok();

    // Set transcribed text
    clipboard.set_text(text)?;

    // Small delay to ensure clipboard is updated
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Simulate Cmd+V
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.key(Key::Meta, Direction::Press)?;
    enigo.key(Key::Unicode('v'), Direction::Click)?;
    enigo.key(Key::Meta, Direction::Release)?;

    // Restore previous clipboard after a delay
    if let Some(prev) = previous {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(&prev);
            }
        });
    }

    Ok(())
}
