use crate::convert::LayoutPair;
use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum TextOpError {
    #[error("clipboard error: {0}")]
    Clipboard(String),
    #[error("keyboard error: {0}")]
    Keyboard(String),
    #[error("no text to convert")]
    Empty,
}

fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}

fn new_enigo() -> Result<Enigo, TextOpError> {
    Enigo::new(&Settings::default()).map_err(|e| TextOpError::Keyboard(e.to_string()))
}

/// Wait until physical modifier keys are released, then force-release via SendInput.
fn settle_modifiers(enigo: &mut Enigo) -> Result<(), TextOpError> {
    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline {
        if !modifiers_physically_down() {
            break;
        }
        sleep_ms(10);
    }

    for key in [
        Key::Control,
        Key::Shift,
        Key::Alt,
        #[cfg(target_os = "macos")]
        Key::Meta,
        #[cfg(windows)]
        Key::LControl,
        #[cfg(windows)]
        Key::RControl,
    ] {
        let _ = enigo.key(key, Release);
    }
    sleep_ms(10);
    Ok(())
}

#[cfg(windows)]
fn modifiers_physically_down() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

    const VK_SHIFT: i32 = 0x10;
    const VK_CONTROL: i32 = 0x11;
    const VK_MENU: i32 = 0x12; // Alt
    const VK_LWIN: i32 = 0x5B;
    const VK_RWIN: i32 = 0x5C;

    [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
        .into_iter()
        .any(|vk| unsafe { GetAsyncKeyState(vk) as u16 & 0x8000 != 0 })
}

#[cfg(target_os = "macos")]
fn modifiers_physically_down() -> bool {
    false
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn modifiers_physically_down() -> bool {
    false
}

fn copy_modifier() -> Key {
    #[cfg(target_os = "macos")]
    {
        Key::Meta
    }
    #[cfg(not(target_os = "macos"))]
    {
        Key::Control
    }
}

fn chord_copy(enigo: &mut Enigo) -> Result<(), TextOpError> {
    let m = copy_modifier();
    enigo
        .key(m, Press)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    enigo
        .key(Key::C, Click)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    enigo
        .key(m, Release)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    Ok(())
}

fn chord_paste(enigo: &mut Enigo) -> Result<(), TextOpError> {
    let m = copy_modifier();
    enigo
        .key(m, Press)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    enigo
        .key(Key::V, Click)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    enigo
        .key(m, Release)
        .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    Ok(())
}

fn select_previous_word(enigo: &mut Enigo) -> Result<(), TextOpError> {
    #[cfg(target_os = "macos")]
    {
        enigo
            .key(Key::Alt, Press)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Shift, Press)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::LeftArrow, Click)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Shift, Release)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Alt, Release)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo
            .key(Key::Control, Press)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Shift, Press)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::LeftArrow, Click)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Shift, Release)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
        enigo
            .key(Key::Control, Release)
            .map_err(|e| TextOpError::Keyboard(e.to_string()))?;
    }
    Ok(())
}

fn wait_clipboard_changed(
    clipboard: &mut Clipboard,
    marker: &str,
    timeout_ms: u64,
) -> Result<String, TextOpError> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match clipboard.get_text() {
            Ok(text) if !text.is_empty() && text != marker => return Ok(text),
            Ok(_) => {}
            Err(e) => return Err(TextOpError::Clipboard(e.to_string())),
        }
        if Instant::now() >= deadline {
            return Err(TextOpError::Empty);
        }
        sleep_ms(5);
    }
}

fn restore_clipboard_later(previous: Option<String>) {
    thread::spawn(move || {
        sleep_ms(80);
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Some(prev) = previous {
                let _ = clipboard.set_text(prev);
            } else {
                let _ = clipboard.clear();
            }
        }
    });
}

/// Copy → convert → paste. Caller must already have settled modifiers.
fn clipboard_roundtrip(enigo: &mut Enigo, pair: &LayoutPair) -> Result<String, TextOpError> {
    let mut clipboard =
        Clipboard::new().map_err(|e| TextOpError::Clipboard(e.to_string()))?;
    let previous = clipboard.get_text().ok();

    let marker = format!("__ls_{}__", std::process::id());
    clipboard
        .set_text(marker.clone())
        .map_err(|e| TextOpError::Clipboard(e.to_string()))?;

    chord_copy(enigo)?;
    let copied = match wait_clipboard_changed(&mut clipboard, &marker, 80) {
        Ok(text) => text,
        Err(err) => {
            if let Some(prev) = previous {
                let _ = clipboard.set_text(prev);
            }
            return Err(err);
        }
    };

    let converted = pair.convert(&copied);
    if converted == copied {
        if let Some(prev) = previous {
            let _ = clipboard.set_text(prev);
        }
        return Ok(converted);
    }

    clipboard
        .set_text(&converted)
        .map_err(|e| TextOpError::Clipboard(e.to_string()))?;
    sleep_ms(10);
    chord_paste(enigo)?;
    // Don't block UX waiting for paste to consume clipboard — restore in background.
    restore_clipboard_later(previous);

    Ok(converted)
}

pub fn convert_selection(pair: &LayoutPair) -> Result<String, TextOpError> {
    let mut enigo = new_enigo()?;
    settle_modifiers(&mut enigo)?;
    clipboard_roundtrip(&mut enigo, pair)
}

pub fn convert_last_word(pair: &LayoutPair) -> Result<String, TextOpError> {
    let mut enigo = new_enigo()?;
    settle_modifiers(&mut enigo)?;
    select_previous_word(&mut enigo)?;
    sleep_ms(25);
    clipboard_roundtrip(&mut enigo, pair)
}
