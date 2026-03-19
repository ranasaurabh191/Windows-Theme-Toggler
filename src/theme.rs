use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendNotifyMessageW, WM_SETTINGCHANGE,
};

const REG_PATH: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

pub fn get_current_theme() -> Theme {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(REG_PATH, KEY_READ)
        .expect("Failed to open theme registry key");

    let value: u32 = key.get_value("AppsUseLightTheme").unwrap_or(1);

    if value == 0 {
        Theme::Dark
    } else {
        Theme::Light
    }
}

pub fn set_theme(theme: Theme) {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(REG_PATH, KEY_WRITE)
        .expect("Failed to open theme registry key for writing");

    let value: u32 = match theme {
        Theme::Dark  => 0,
        Theme::Light => 1,
    };

    key.set_value("AppsUseLightTheme", &value).unwrap();
    key.set_value("SystemUsesLightTheme", &value).unwrap();

    // Broadcast to Explorer so taskbar + Start menu update immediately
    let param: Vec<u16> = "ImmersiveColorSet\0"
        .encode_utf16()
        .collect();

    unsafe {
        SendNotifyMessageW(
            0xffff as _,         // HWND_BROADCAST
            WM_SETTINGCHANGE,
            0,
            param.as_ptr() as _,
        );
    }
}

pub fn toggle() -> Theme {
    let next = match get_current_theme() {
        Theme::Dark  => Theme::Light,
        Theme::Light => Theme::Dark,
    };
    set_theme(next);
    next
}