# Simple Layout Switcher

Lightweight [Punto Switcher](https://yandex.ru/soft/punto/)-style utility: fix text typed in the wrong keyboard layout with two hotkeys. No auto-detection noise — only manual convert when you ask for it.

Легковесная замена Punto Switcher: два hotkey для конвертации текста между раскладками. Без автопереключения — только по запросу.

**Windows** · **macOS** · MIT · built with [Tauri](https://tauri.app/) + Rust

---

## Features / Возможности

- Convert the **last word** (Punto-style)
- Convert the **selected text**
- Optionally switch the **system keyboard layout** after convert
- Configurable hotkeys and EN↔RU layout pair
- System tray + optional autostart

## Default hotkeys / Горячие клавиши

| Action | Windows | macOS |
| --- | --- | --- |
| Last word | `Pause` | `F11` |
| Selection | `Shift+Pause` | `Shift+F11` |

On MacBooks there is no Pause key, so defaults follow Yandex’s laptop Break fallback (`F11`). Classic Punto for Mac used bare **Option** — you can assign any combo in Settings (click the field and press the keys, then Save).

## Install / Установка

### Prebuilt (Windows)

Download the latest installer from [Releases](https://github.com/eminemgis/simple-layout-switcher/releases).

macOS builds will appear when available; for now build from source on a Mac.

### From source

Requirements: Node.js, Rust stable, platform C++ toolchain (MSVC Build Tools on Windows / Xcode on macOS).

```bash
git clone https://github.com/eminemgis/simple-layout-switcher.git
cd simple-layout-switcher
npm install
npm run tauri build
```

Dev mode:

```bash
npm run tauri dev
```

## Usage / Использование

1. Start the app (tray icon).
2. In Notepad or any text field type `ghbdtn`.
3. Press **Pause** (Windows) → becomes `привет`.
4. Select text → **Shift+Pause** to convert the selection.
5. Open Settings from the tray to change hotkeys, layout pair, system-layout switch, and autostart.

Closing the settings window keeps the app running in the tray.

## Platform notes / Замечания

- **macOS**: grant **Accessibility** (and Input Monitoring if prompted).
- **Windows**: works without admin in normal apps. Elevated (Administrator) apps may ignore synthetic input unless this app is also elevated.
- Password fields and some games can block clipboard / key simulation — same limitation as other layout tools.

## How it works / Как работает

1. Global hotkey fires.
2. App copies the selection (or selects the previous word, then copies).
3. Characters are remapped EN↔RU by physical key positions.
4. Converted text is pasted back.
5. If enabled, the OS input language is switched to match the result.

## License

[MIT](LICENSE)
