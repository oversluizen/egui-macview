//! WebAssembly (`wasm32`) FFI bridge for `MobileWebSession` (`REQ-013-02O`).

#![cfg(target_arch = "wasm32")]
#![allow(unsafe_code)]

use super::mobile_web_server::MobileWebSession;
use std::{
    fmt::Write,
    sync::{Mutex, OnceLock},
};

struct WasmRt {
    session: MobileWebSession,
    cmd_log: String,
    last_frame: Vec<u8>,
    last_atlas: Vec<u8>,
}

fn with_rt<R>(f: impl FnOnce(&mut WasmRt) -> R) -> Option<R> {
    static RT: OnceLock<Mutex<WasmRt>> = OnceLock::new();
    let mtx = RT.get_or_init(|| {
        let session = MobileWebSession::new();
        let last_atlas = session.encode_atlas();
        Mutex::new(WasmRt { session, cmd_log: String::new(), last_frame: Vec::new(), last_atlas })
    });
    mtx.lock().ok().map(|mut g| f(&mut g))
}

#[no_mangle]
pub extern "C" fn wasm_init() {
    let _ = with_rt(|_| {});
}

#[no_mangle]
pub extern "C" fn wasm_push_pointer_move(x: f32, y: f32) {
    let _ = with_rt(|rt| writeln!(rt.cmd_log, "M {x:.1} {y:.1}"));
}

#[no_mangle]
pub extern "C" fn wasm_push_pointer_button(x: f32, y: f32, pressed: i32) {
    let _ = with_rt(|rt| writeln!(rt.cmd_log, "{} {x:.1} {y:.1}", if pressed != 0 { "D" } else { "U" }));
}

#[no_mangle]
pub extern "C" fn wasm_push_wheel(dx: f32, dy: f32) {
    let _ = with_rt(|rt| writeln!(rt.cmd_log, "W {dx:.1} {dy:.1}"));
}

#[no_mangle]
pub extern "C" fn wasm_push_key(key_code: i32) {
    if let Some(k) = match key_code {
        8 => Some("Backspace"),
        13 => Some("Enter"),
        38 => Some("ArrowUp"),
        40 => Some("ArrowDown"),
        27 => Some("Escape"),
        _ => None,
    } {
        let _ = with_rt(|rt| writeln!(rt.cmd_log, "K {k}"));
    }
}

#[no_mangle]
pub extern "C" fn wasm_push_char(codepoint: u32) {
    if let Some(ch) = char::from_u32(codepoint) {
        let _ = with_rt(|rt| writeln!(rt.cmd_log, "T {ch}"));
    }
}

#[no_mangle]
pub extern "C" fn wasm_render_frame(w: f32, h: f32, dpr: f32, time_secs: f64) -> usize {
    with_rt(|rt| {
        let batch = format!("V {w:.1} {h:.1} {dpr:.2}\nC {time_secs:.4}\n{}", rt.cmd_log);
        rt.cmd_log.clear();
        let prev_ver = rt.session.atlas_version;
        rt.last_frame = rt.session.encode_frame(w, h, dpr, &batch);
        if rt.session.atlas_version != prev_ver || rt.last_atlas.is_empty() {
            rt.last_atlas = rt.session.encode_atlas();
        }
        rt.last_frame.len()
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_frame_ptr() -> usize {
    with_rt(|rt| rt.last_frame.as_ptr() as usize).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_atlas_ptr() -> usize {
    with_rt(|rt| rt.last_atlas.as_ptr() as usize).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_atlas_len() -> usize {
    with_rt(|rt| rt.last_atlas.len()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_media_state() -> u32 {
    with_rt(|rt| {
        let on_avkit = u32::from(rt.session.app.active_screen == super::types::GalleryScreen::AvKitMediaStudio);
        let local_disk = u32::from(
            rt.session.app.active_screen == super::types::GalleryScreen::FinderDataBrowser
                && rt.session.app.center.finder_current_dir.is_some(),
        );
        on_avkit
            | (u32::from(rt.session.app.center.media_playing) << 1)
            | (local_disk << 2)
            | (((rt.session.app.center.codec_preset as u32) & 0xF) << 4)
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn wasm_media_rect(idx: i32) -> f32 {
    with_rt(|rt| {
        let (r, c) = (rt.session.app.center.last_video_viewport_rect, &rt.session.app.center);
        [
            r.min.x,
            r.min.y,
            r.width().max(0.0),
            r.height().max(0.0),
            c.media_gain.clamp(0.0, 1.0),
            c.media_speed.clamp(0.25, 2.0),
            c.media_in_point as f32,
            c.media_out_point as f32,
            c.media_playhead as f32,
        ]
        .get(idx as usize)
        .copied()
        .unwrap_or(0.0)
    })
    .unwrap_or(0.0)
}
