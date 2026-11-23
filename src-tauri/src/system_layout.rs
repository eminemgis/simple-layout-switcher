//! Switch the OS input language to match converted text.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLang {
    English,
    Russian,
}

/// Guess the language of `text` by letter script (Cyrillic vs Latin).
pub fn detect_lang(text: &str) -> Option<InputLang> {
    let mut cyrillic = 0usize;
    let mut latin = 0usize;
    for c in text.chars() {
        if c.is_ascii_alphabetic() {
            latin += 1;
        } else if ('\u{0400}'..='\u{04FF}').contains(&c) {
            cyrillic += 1;
        }
    }
    if cyrillic == 0 && latin == 0 {
        return None;
    }
    if cyrillic >= latin {
        Some(InputLang::Russian)
    } else {
        Some(InputLang::English)
    }
}

pub fn activate(lang: InputLang) {
    if let Err(err) = activate_inner(lang) {
        eprintln!("lang-switcher: system layout switch failed: {err}");
    } else {
        eprintln!("lang-switcher: system layout -> {lang:?}");
    }
}

#[cfg(windows)]
fn activate_inner(lang: InputLang) -> Result<(), String> {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        ActivateKeyboardLayout, KLF_SETFORPROCESS,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, PostMessageW, WM_INPUTLANGCHANGEREQUEST,
    };

    let lang_id: u16 = match lang {
        InputLang::English => 0x0409, // en-US
        InputLang::Russian => 0x0419, // ru-RU
    };

    // Prefer an already-loaded layout matching the language.
    let hkl = match find_loaded_hkl(lang_id) {
        Some(h) => h,
        None => load_hkl(lang_id)?,
    };

    let hwnd: HWND = unsafe { GetForegroundWindow() };
    if !hwnd.is_null() {
        let ok = unsafe {
            PostMessageW(
                hwnd,
                WM_INPUTLANGCHANGEREQUEST,
                0,
                hkl as isize,
            )
        };
        if ok == 0 {
            // Fallback: activate for this process / thread.
            let _ = unsafe { ActivateKeyboardLayout(hkl, KLF_SETFORPROCESS) };
        }
    } else {
        let _ = unsafe { ActivateKeyboardLayout(hkl, KLF_SETFORPROCESS) };
    }
    Ok(())
}

#[cfg(windows)]
fn find_loaded_hkl(lang_id: u16) -> Option<windows_sys::Win32::UI::Input::KeyboardAndMouse::HKL> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayoutList;

    let count = unsafe { GetKeyboardLayoutList(0, std::ptr::null_mut()) };
    if count <= 0 {
        return None;
    }
    let mut list = vec![std::ptr::null_mut(); count as usize];
    let got = unsafe { GetKeyboardLayoutList(count, list.as_mut_ptr()) };
    if got <= 0 {
        return None;
    }
    list.into_iter().find(|&hkl| {
        if hkl.is_null() {
            return false;
        }
        // Low word of HKL is the LANGID.
        ((hkl as usize) & 0xFFFF) as u16 == lang_id
    })
}

#[cfg(windows)]
fn load_hkl(
    lang_id: u16,
) -> Result<windows_sys::Win32::UI::Input::KeyboardAndMouse::HKL, String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{LoadKeyboardLayoutW, KLF_ACTIVATE};

    let name: Vec<u16> = format!("{lang_id:08X}")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let hkl = unsafe { LoadKeyboardLayoutW(name.as_ptr(), KLF_ACTIVATE) };
    if hkl.is_null() {
        Err(format!("LoadKeyboardLayoutW({lang_id:04X}) failed"))
    } else {
        Ok(hkl)
    }
}

#[cfg(target_os = "macos")]
fn activate_inner(lang: InputLang) -> Result<(), String> {
    // Select by BCP-47 language via Carbon TIS (linked through CoreFoundation symbols
    // available when the process runs as a normal macOS app).
    use std::ffi::c_void;
    use std::ptr;

    type CFStringRef = *const c_void;
    type TISInputSourceRef = *mut c_void;
    type OSStatus = i32;

    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn TISCopyInputSourceForLanguage(language: CFStringRef) -> TISInputSourceRef;
        fn TISSelectInputSource(source: TISInputSourceRef) -> OSStatus;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(
            alloc: *const c_void,
            c_str: *const i8,
            encoding: u32,
        ) -> CFStringRef;
        fn CFRelease(cf: *const c_void);
    }

    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    let code = match lang {
        InputLang::English => "en\0",
        InputLang::Russian => "ru\0",
    };

    unsafe {
        let cf_lang = CFStringCreateWithCString(
            ptr::null(),
            code.as_ptr() as *const i8,
            K_CF_STRING_ENCODING_UTF8,
        );
        if cf_lang.is_null() {
            return Err("CFStringCreateWithCString failed".into());
        }
        let source = TISCopyInputSourceForLanguage(cf_lang);
        CFRelease(cf_lang);
        if source.is_null() {
            return Err(format!("No input source for {lang:?}"));
        }
        let status = TISSelectInputSource(source);
        CFRelease(source as *const c_void);
        if status != 0 {
            return Err(format!("TISSelectInputSource status={status}"));
        }
    }
    Ok(())
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn activate_inner(_lang: InputLang) -> Result<(), String> {
    Err("system layout switch is not supported on this OS".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_russian() {
        assert_eq!(detect_lang("привет"), Some(InputLang::Russian));
    }

    #[test]
    fn detects_english() {
        assert_eq!(detect_lang("hello"), Some(InputLang::English));
    }
}
