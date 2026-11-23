mod convert;
mod hotkeys;
mod settings;
mod system_layout;
mod text_ops;

use hotkeys::AppState;
use settings::{AppSettings, LayoutPairInfo};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
#[cfg(desktop)]
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn list_layout_pairs() -> Vec<LayoutPairInfo> {
    convert::available_pairs()
        .into_iter()
        .map(|p| LayoutPairInfo {
            id: p.id.to_string(),
            label: p.label.to_string(),
        })
        .collect()
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    #[cfg(desktop)]
    {
        let autostart = app.autolaunch();
        if settings.autostart {
            let _ = autostart.enable();
        } else {
            let _ = autostart.disable();
        }
    }
    hotkeys::apply_settings(&app, settings)
}

#[tauri::command]
fn convert_text(state: tauri::State<'_, AppState>, text: String) -> String {
    state.layout_pair().convert(&text)
}

#[tauri::command]
fn trigger_convert_word(app: tauri::AppHandle) -> Result<(), String> {
    hotkeys::trigger(&app, hotkeys::ConvertMode::Word);
    Ok(())
}

#[tauri::command]
fn trigger_convert_selection(app: tauri::AppHandle) -> Result<(), String> {
    hotkeys::trigger(&app, hotkeys::ConvertMode::Selection);
    Ok(())
}

fn show_settings_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial = settings::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    hotkeys::handle_shortcut_event(app, shortcut, event.state);
                })
                .build(),
        )
        .manage(AppState::new(initial))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            list_layout_pairs,
            convert_text,
            trigger_convert_word,
            trigger_convert_selection
        ])
        .setup(|app| {
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Lang Switcher")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "settings" => show_settings_window(app),
                    _ => {}
                })
                .build(app)?;

            let hidden = std::env::args().any(|a| a == "--hidden");
            if let Some(window) = app.get_webview_window("main") {
                if hidden {
                    let _ = window.hide();
                }
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app_handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            if let Err(err) = hotkeys::register_from_settings(app.handle()) {
                eprintln!("lang-switcher: hotkey register failed: {err}");
            }

            #[cfg(desktop)]
            {
                let settings = app.state::<AppState>().settings.lock().clone();
                let autostart = app.autolaunch();
                if settings.autostart {
                    let _ = autostart.enable();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running lang-switcher");
}
