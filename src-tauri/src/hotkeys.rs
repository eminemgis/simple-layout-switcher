use crate::convert::{self, LayoutPair};
use crate::settings::AppSettings;
use crate::system_layout;
use crate::text_ops;
use parking_lot::Mutex;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub busy: Mutex<bool>,
}

impl AppState {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings: Mutex::new(settings),
            busy: Mutex::new(false),
        }
    }

    pub fn layout_pair(&self) -> LayoutPair {
        let id = self.settings.lock().layout_pair.clone();
        convert::pair_by_id(&id).unwrap_or_else(convert::en_ru)
    }
}

#[derive(Clone, Copy)]
pub enum ConvertMode {
    Word,
    Selection,
}

pub fn trigger<R: Runtime>(app: &AppHandle<R>, mode: ConvertMode) {
    let handle = app.clone();
    thread::spawn(move || {
        // Give UI focus time to leave the settings window if user clicked a test button.
        thread::sleep(Duration::from_millis(250));
        run_convert(&handle, mode);
    });
}

fn run_convert<R: Runtime>(app: &AppHandle<R>, mode: ConvertMode) {
    let state = app.state::<AppState>();
    {
        let mut busy = state.busy.lock();
        if *busy {
            return;
        }
        *busy = true;
    }

    let pair = state.layout_pair();
    let switch_layout = state.settings.lock().switch_system_layout;
    let result = match mode {
        ConvertMode::Word => text_ops::convert_last_word(&pair),
        ConvertMode::Selection => text_ops::convert_selection(&pair),
    };

    match result {
        Ok(text) => {
            eprintln!("lang-switcher: converted -> {text}");
            if switch_layout {
                if let Some(lang) = system_layout::detect_lang(&text) {
                    system_layout::activate(lang);
                }
            }
        }
        Err(err) => eprintln!("lang-switcher: {err}"),
    }

    *state.busy.lock() = false;
}

fn parse_shortcut(raw: &str, kind: &str) -> Result<Shortcut, String> {
    Shortcut::from_str(raw).map_err(|e| format!("Invalid {kind} hotkey '{raw}': {e}"))
}

pub fn unregister_all<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())
}

pub fn register_from_settings<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let settings = app.state::<AppState>().settings.lock().clone();
    let word = parse_shortcut(&settings.hotkey_word, "word")?;
    let selection = parse_shortcut(&settings.hotkey_selection, "selection")?;

    if word.id() == selection.id() {
        return Err("Hotkeys must be different".into());
    }

    // Drop previous bindings first; ignore "not registered" style failures.
    let _ = unregister_all(app);
    thread::sleep(Duration::from_millis(50));

    app.global_shortcut()
        .register(word)
        .map_err(|e| format!("Could not register '{}': {e}", settings.hotkey_word))?;
    app.global_shortcut()
        .register(selection)
        .map_err(|e| format!("Could not register '{}': {e}", settings.hotkey_selection))?;

    eprintln!(
        "lang-switcher: hotkeys ready (word={}, selection={})",
        settings.hotkey_word, settings.hotkey_selection
    );
    Ok(())
}

pub fn handle_shortcut_event<R: Runtime>(
    app: &AppHandle<R>,
    shortcut: &Shortcut,
    state: ShortcutState,
) {
    if state != ShortcutState::Pressed {
        return;
    }

    let settings = app.state::<AppState>().settings.lock().clone();
    let Ok(word) = parse_shortcut(&settings.hotkey_word, "word") else {
        return;
    };
    let Ok(selection) = parse_shortcut(&settings.hotkey_selection, "selection") else {
        return;
    };

    let mode = if shortcut.id() == word.id() {
        ConvertMode::Word
    } else if shortcut.id() == selection.id() {
        ConvertMode::Selection
    } else {
        return;
    };

    let handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        run_convert(&handle, mode);
    });
}

pub fn apply_settings<R: Runtime>(
    app: &AppHandle<R>,
    settings: AppSettings,
) -> Result<(), String> {
    crate::settings::save(&settings)?;
    *app.state::<AppState>().settings.lock() = settings;
    register_from_settings(app)?;
    Ok(())
}
