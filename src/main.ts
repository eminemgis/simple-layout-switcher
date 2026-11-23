import { invoke } from "@tauri-apps/api/core";

type AppSettings = {
  hotkeyWord: string;
  hotkeySelection: string;
  layoutPair: string;
  autostart: boolean;
  switchSystemLayout: boolean;
};

type LayoutPairInfo = {
  id: string;
  label: string;
};

const el = {
  word: () => document.querySelector<HTMLInputElement>("#hotkey-word")!,
  selection: () => document.querySelector<HTMLInputElement>("#hotkey-selection")!,
  layout: () => document.querySelector<HTMLSelectElement>("#layout-pair")!,
  autostart: () => document.querySelector<HTMLInputElement>("#autostart")!,
  switchSystemLayout: () =>
    document.querySelector<HTMLInputElement>("#switch-system-layout")!,
  previewIn: () => document.querySelector<HTMLInputElement>("#preview-input")!,
  previewOut: () => document.querySelector<HTMLElement>("#preview-out")!,
  status: () => document.querySelector<HTMLElement>("#status")!,
};

function setStatus(message: string, isError = false) {
  const node = el.status();
  node.textContent = message;
  node.style.color = isError ? "#b42318" : "#5b6472";
}

function codeToToken(code: string, key: string): string | null {
  if (code.startsWith("Key") && code.length === 4) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (/^F\d{1,2}$/.test(code)) return code;
  const special: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Escape: "Escape",
    Tab: "Tab",
    Backspace: "Backspace",
    Pause: "Pause",
    ScrollLock: "ScrollLock",
    Insert: "Insert",
    Delete: "Delete",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    ArrowUp: "ArrowUp",
    ArrowDown: "ArrowDown",
    ArrowLeft: "ArrowLeft",
    ArrowRight: "ArrowRight",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Semicolon: ";",
    Quote: "'",
    Backquote: "`",
    Backslash: "\\",
    Comma: ",",
    Period: ".",
    Slash: "/",
  };
  if (special[code]) return special[code];
  if (key.length === 1) return key.toUpperCase();
  return null;
}

function eventToHotkey(e: KeyboardEvent): string | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return null;
  const token = codeToToken(e.code, e.key);
  if (!token) return null;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Meta");
  parts.push(token);
  return parts.join("+");
}

function bindHotkeyCapture(input: HTMLInputElement, label: string) {
  input.readOnly = true;
  input.placeholder = "Click, then press shortcut";

  input.addEventListener("focus", () => {
    input.classList.add("capturing");
    setStatus(`Press a shortcut for ${label}…`);
  });

  input.addEventListener("blur", () => {
    input.classList.remove("capturing");
  });

  input.addEventListener("keydown", (e) => {
    e.preventDefault();
    e.stopPropagation();

    if (e.key === "Escape") {
      input.blur();
      setStatus("Cancelled");
      return;
    }

    if (e.key === "Backspace" || e.key === "Delete") {
      input.value = "";
      setStatus(`${label}: cleared`);
      return;
    }

    const hotkey = eventToHotkey(e);
    if (!hotkey) return;
    input.value = hotkey;
    setStatus(`${label}: ${hotkey}`);
    input.blur();
  });
}

function readForm(): AppSettings {
  return {
    hotkeyWord: el.word().value.trim(),
    hotkeySelection: el.selection().value.trim(),
    layoutPair: el.layout().value,
    autostart: el.autostart().checked,
    switchSystemLayout: el.switchSystemLayout().checked,
  };
}

function fillForm(settings: AppSettings) {
  el.word().value = settings.hotkeyWord;
  el.selection().value = settings.hotkeySelection;
  el.layout().value = settings.layoutPair;
  el.autostart().checked = settings.autostart;
  el.switchSystemLayout().checked = settings.switchSystemLayout ?? true;
}

async function load() {
  const [settings, pairs] = await Promise.all([
    invoke<AppSettings>("get_settings"),
    invoke<LayoutPairInfo[]>("list_layout_pairs"),
  ]);

  const select = el.layout();
  select.innerHTML = "";
  for (const pair of pairs) {
    const opt = document.createElement("option");
    opt.value = pair.id;
    opt.textContent = pair.label;
    select.appendChild(opt);
  }

  fillForm(settings);
  setStatus("Ready — click a hotkey field to change it");
}

async function save() {
  const settings = readForm();
  if (!settings.hotkeyWord || !settings.hotkeySelection) {
    setStatus("Both hotkeys are required", true);
    return;
  }
  if (settings.hotkeyWord === settings.hotkeySelection) {
    setStatus("Hotkeys must be different", true);
    return;
  }
  try {
    await invoke("save_settings", { settings });
    setStatus("Saved — hotkeys updated");
  } catch (err) {
    setStatus(String(err), true);
  }
}

async function preview() {
  const text = el.previewIn().value;
  const converted = await invoke<string>("convert_text", { text });
  el.previewOut().textContent = converted;
}

window.addEventListener("DOMContentLoaded", () => {
  bindHotkeyCapture(el.word(), "Last word");
  bindHotkeyCapture(el.selection(), "Selection");

  document.querySelector("#save-btn")?.addEventListener("click", () => {
    void save();
  });
  document.querySelector("#preview-btn")?.addEventListener("click", () => {
    void preview();
  });
  document.querySelector("#test-word-btn")?.addEventListener("click", () => {
    setStatus("Switch to target app… testing last word");
    void invoke("trigger_convert_word").catch((err) => setStatus(String(err), true));
  });
  document.querySelector("#test-sel-btn")?.addEventListener("click", () => {
    setStatus("Switch to target app… testing selection");
    void invoke("trigger_convert_selection").catch((err) =>
      setStatus(String(err), true),
    );
  });
  el.previewIn().addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      void preview();
    }
  });
  void load().catch((err) => setStatus(String(err), true));
});
