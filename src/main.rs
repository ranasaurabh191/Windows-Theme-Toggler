#![windows_subsystem = "windows"]

mod theme;

use std::time::{Duration, Instant};
use image::{RgbaImage, imageops};
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    MouseButton, MouseButtonState,
    menu::{Menu, MenuItem, MenuEvent},
};
use theme::{Theme, toggle, get_current_theme};

use windows_sys::Win32::UI::WindowsAndMessaging::{
    PeekMessageW, TranslateMessage, DispatchMessageW,
    MSG, PM_REMOVE,
};

const FRAME_COUNT: usize = 8;
const FRAME_MS:    u64   = 16;

// Icons are baked INTO the exe at compile time — no external files needed
static DARK_PNG:  &[u8] = include_bytes!("../assets/dark.png");
static LIGHT_PNG: &[u8] = include_bytes!("../assets/light.png");

fn load_rgba_from_bytes(bytes: &[u8]) -> RgbaImage {
    let img = image::load_from_memory(bytes)
        .expect("Failed to decode embedded icon")
        .to_rgba8();
    imageops::resize(&img, 32, 32, imageops::FilterType::Lanczos3)
}

fn to_tray_icon(img: &RgbaImage) -> tray_icon::Icon {
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.as_raw().clone(), w, h)
        .expect("Failed to build tray icon")
}

fn blend(from: &RgbaImage, to: &RgbaImage, t: f32) -> RgbaImage {
    let (w, h) = from.dimensions();
    let mut out = RgbaImage::new(w, h);
    let t = t * t * (3.0 - 2.0 * t);
    for (x, y, px) in out.enumerate_pixels_mut() {
        let a = from.get_pixel(x, y).0;
        let b = to.get_pixel(x, y).0;
        *px = image::Rgba([
            (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
            (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
            (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
            (a[3] as f32 + (b[3] as f32 - a[3] as f32) * t) as u8,
        ]);
    }
    out
}

fn icon_for_theme<'a>(theme: &Theme, dark: &'a RgbaImage, light: &'a RgbaImage) -> &'a RgbaImage {
    match theme {
        Theme::Dark  => light,
        Theme::Light => dark,
    }
}

enum AnimDir { ToDark, ToLight }

struct Anim {
    dir:       AnimDir,
    frame:     usize,
    last_tick: Instant,
}

fn main() {
    // Load icons from embedded bytes — no files needed on disk
    let dark_img  = load_rgba_from_bytes(DARK_PNG);
    let light_img = load_rgba_from_bytes(LIGHT_PNG);

    let menu      = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&quit_item).unwrap();
    let quit_id = quit_item.id().clone();

    let current      = get_current_theme();
    let initial_icon = to_tray_icon(icon_for_theme(&current, &dark_img, &light_img));

    let mut tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("Click to switch theme")
        .with_icon(initial_icon)
        .build()
        .expect("Failed to create tray icon");

    let tray_rx = TrayIconEvent::receiver();
    let menu_rx = MenuEvent::receiver();
    let mut anim: Option<Anim> = None;

    unsafe {
        let mut msg = std::mem::zeroed::<MSG>();

        loop {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            while let Ok(event) = tray_rx.try_recv() {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event {
                    let new_theme = toggle();
                    let dir = match new_theme {
                        Theme::Dark  => AnimDir::ToLight,
                        Theme::Light => AnimDir::ToDark,
                    };
                    anim = Some(Anim { dir, frame: 0, last_tick: Instant::now() });
                }
            }

            while let Ok(event) = menu_rx.try_recv() {
                if event.id == quit_id {
                    std::process::exit(0);
                }
            }

            if let Some(ref mut a) = anim {
                if a.last_tick.elapsed() >= Duration::from_millis(FRAME_MS) {
                    if a.frame <= FRAME_COUNT {
                        let t = a.frame as f32 / FRAME_COUNT as f32;
                        let frame = match a.dir {
                            AnimDir::ToLight => blend(&dark_img, &light_img, t),
                            AnimDir::ToDark  => blend(&light_img, &dark_img, t),
                        };
                        tray.set_icon(Some(to_tray_icon(&frame))).unwrap();
                        a.frame    += 1;
                        a.last_tick = Instant::now();
                    } else {
                        anim = None;
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}