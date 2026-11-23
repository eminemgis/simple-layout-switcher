use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Hotkey for converting the last typed word.
    pub hotkey_word: String,
    /// Hotkey for converting the current selection.
    pub hotkey_selection: String,
    /// Layout pair id, e.g. "en-ru".
    pub layout_pair: String,
    pub autostart: bool,
    /// After converting text, also switch the OS keyboard layout.
    #[serde(default = "default_true")]
    pub switch_system_layout: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        // Match Punto Switcher defaults per platform.
        // Windows: Pause / Shift+Pause.
        // macOS: no Pause on MacBooks — F11 / Shift+F11 (Yandex's laptop Break fallback).
        #[cfg(target_os = "macos")]
        {
            Self {
                hotkey_word: "F11".into(),
                hotkey_selection: "Shift+F11".into(),
                layout_pair: "en-ru".into(),
                autostart: false,
                switch_system_layout: true,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                hotkey_word: "Pause".into(),
                hotkey_selection: "Shift+Pause".into(),
                layout_pair: "en-ru".into(),
                autostart: false,
                switch_system_layout: true,
            }
        }
    }
}

fn config_path() -> Result<PathBuf, String> {
    let dir = dirs::config_dir()
        .ok_or_else(|| "Could not resolve config directory".to_string())?
        .join("lang-switcher");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load() -> AppSettings {
    let Ok(path) = config_path() else {
        return AppSettings::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = config_path()?;
    let raw = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutPairInfo {
    pub id: String,
    pub label: String,
}
