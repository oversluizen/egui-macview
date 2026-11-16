//! Built-in Zero-Dependency Mobile & Tablet WebGL2 Browser Server (`REQ-013-02O`).
//!
//! Streams `egui::Context::tessellate` triangle meshes (`MWGL` binary protocol) and the
//! RGBA8 font/symbol atlas over HTTP (`0.0.0.0:8080`) so any iPhone, iPad, or Android
//! tablet on the local Wi-Fi network can interact with `macview_gallery` in Safari or Chrome.

use super::setup_system_or_custom_fonts;
use crate::{GalleryApp, GalleryScreen};
use egui::{pos2, vec2, Align, Context, Event, Key, Layout, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, Ui};
use egui_macview::{theme::MacTheme, MacButtonCell, MacSegmentedControl};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    io::{Read, Write},
    net::{TcpListener, UdpSocket},
    sync::{Arc, Mutex},
};

#[cfg(not(target_arch = "wasm32"))]
const MOBILE_WEB_HTML: &str = include_str!("mobile_web_html.html");

/// Interactive headless `GalleryApp` session driving the mobile/tablet WebGL2 `<canvas>`.
pub(crate) struct MobileWebSession {
    pub(crate) ctx: Context,
    pub(crate) app: GalleryApp,
    pub(crate) atlas_rgba: Vec<u8>,
    pub(crate) atlas_width: u32,
    pub(crate) atlas_height: u32,
    pub(crate) atlas_version: u32,
    pub(crate) time_secs: f64,
    pub(crate) last_pos: Pos2,
    pub(crate) last_viewport: (f32, f32, f32),
}

impl MobileWebSession {
    pub(crate) fn new() -> Self {
        let ctx = Context::default();
        setup_system_or_custom_fonts(&ctx);
        let mut session = Self {
            ctx,
            app: GalleryApp::new(),
            atlas_rgba: Vec::new(),
            atlas_width: 0,
            atlas_height: 0,
            atlas_version: 0,
            time_secs: 1.0,
            last_pos: pos2(180.0, 200.0),
            last_viewport: (390.0, 844.0, 2.0),
        };
        let _ = session.encode_frame(390.0, 844.0, 2.0, "");
        session
    }

    pub(crate) fn encode_atlas(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.atlas_rgba.len());
        out.extend_from_slice(&self.atlas_width.to_le_bytes());
        out.extend_from_slice(&self.atlas_height.to_le_bytes());
        out.extend_from_slice(&self.atlas_rgba);
        out
    }

    pub(crate) fn encode_frame(&mut self, width_arg: f32, height_arg: f32, dpr_arg: f32, body: &str) -> Vec<u8> {
        if (width_arg - 390.0).abs() > 0.5 || (height_arg - 844.0).abs() > 0.5 {
            self.last_viewport = (width_arg.clamp(280.0, 2560.0), height_arg.clamp(320.0, 2560.0), dpr_arg.clamp(1.0, 3.0));
        }
        let (mut width, mut height, mut dpr) = self.last_viewport;
        let mut event_passes: Vec<Vec<Event>> = vec![Vec::new()];
        let mut has_press_in_current_pass = false;
        let mut client_clock: Option<f64> = None;
        for line in body.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.as_slice() {
                ["V", w, h, d] => {
                    width = w.parse::<f32>().unwrap_or(width).clamp(280.0, 2560.0);
                    height = h.parse::<f32>().unwrap_or(height).clamp(320.0, 2560.0);
                    dpr = d.parse::<f32>().unwrap_or(dpr).clamp(1.0, 3.0);
                    self.last_viewport = (width, height, dpr);
                }
                ["C", t_str] => {
                    if let Ok(t) = t_str.parse::<f64>() {
                        client_clock = Some(t);
                    }
                }
                ["M", x, y] => {
                    let p = pos2(x.parse().unwrap_or(0.0), y.parse().unwrap_or(0.0));
                    self.last_pos = p;
                    if let Some(cur) = event_passes.last_mut() {
                        cur.push(Event::PointerMoved(p));
                    }
                }
                ["D", x, y] => {
                    let p = pos2(x.parse().unwrap_or(0.0), y.parse().unwrap_or(0.0));
                    (self.last_pos, has_press_in_current_pass) = (p, true);
                    if let Some(cur) = event_passes.last_mut() {
                        cur.push(Event::PointerMoved(p));
                        cur.push(Event::PointerButton {
                            pos: p,
                            button: PointerButton::Primary,
                            pressed: true,
                            modifiers: Modifiers::NONE,
                        });
                    }
                }
                ["U", x, y] => {
                    let p = pos2(x.parse().unwrap_or(0.0), y.parse().unwrap_or(0.0));
                    self.last_pos = p;
                    if has_press_in_current_pass {
                        event_passes.push(Vec::new());
                        has_press_in_current_pass = false;
                    }
                    if let Some(cur) = event_passes.last_mut() {
                        cur.push(Event::PointerMoved(p));
                        cur.push(Event::PointerButton {
                            pos: p,
                            button: PointerButton::Primary,
                            pressed: false,
                            modifiers: Modifiers::NONE,
                        });
                    }
                }
                ["W", dx, dy] => {
                    let delta = vec2(dx.parse().unwrap_or(0.0), dy.parse().unwrap_or(0.0));
                    if let Some(cur) = event_passes.last_mut() {
                        cur.push(Event::MouseWheel {
                            unit: MouseWheelUnit::Point,
                            delta,
                            modifiers: Modifiers::NONE,
                            phase: egui::TouchPhase::Move,
                            source: egui::MouseWheelSource::Trackpad,
                        });
                    }
                }
                ["K", k] => {
                    if let Some(key) = match *k {
                        "Backspace" => Some(Key::Backspace),
                        "Enter" => Some(Key::Enter),
                        "ArrowUp" => Some(Key::ArrowUp),
                        "ArrowDown" => Some(Key::ArrowDown),
                        "Escape" => Some(Key::Escape),
                        _ => None,
                    } {
                        if let Some(cur) = event_passes.last_mut() {
                            cur.push(Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
                        }
                    }
                }
                _ if line.starts_with("T ") => {
                    if let Some(cur) = event_passes.last_mut().filter(|_| line.len() > 2) {
                        cur.push(Event::Text(line[2..].to_string()));
                    }
                }
                _ => {}
            }
        }

        let mut last_shapes = Vec::new();
        let mut anim_active = false;
        let mut cursor_code = 0u32;
        if let Some(ct) = client_clock {
            self.time_secs = ct.max(self.time_secs + 0.0005);
        }
        for (pass_idx, events) in event_passes.into_iter().enumerate() {
            if client_clock.is_none() || pass_idx > 0 {
                self.time_secs += 1.0 / 60.0;
            }
            let mut raw = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(width, height))),
                time: Some(self.time_secs),
                focused: true,
                events,
                ..Default::default()
            };
            raw.viewports.entry(egui::ViewportId::ROOT).or_default().native_pixels_per_point = Some(dpr);

            let mut full_output = self.ctx.run_ui(raw, |ui| self.app.render_ui(ui));
            if full_output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .is_some_and(|v| v.repaint_delay <= std::time::Duration::from_millis(100))
            {
                anim_active = true;
            }
            use egui::CursorIcon as C;
            cursor_code = match full_output.platform_output.cursor_icon {
                C::ResizeHorizontal | C::ResizeColumn | C::ResizeEast | C::ResizeWest => 1,
                C::ResizeVertical | C::ResizeRow | C::ResizeNorth | C::ResizeSouth => 2,
                C::ResizeSouthEast | C::ResizeNorthWest | C::ResizeNwSe => 3,
                C::PointingHand => 4,
                C::Text | C::VerticalText => 5,
                _ => 0,
            };
            self.apply_texture_deltas(&full_output.textures_delta);
            full_output.textures_delta.clear();
            last_shapes = full_output.shapes;
        }
        let meshes: Vec<_> = self
            .ctx
            .tessellate(last_shapes, dpr)
            .into_iter()
            .filter_map(|c| match c.primitive {
                egui::epaint::Primitive::Mesh(m)
                    if m.texture_id == egui::TextureId::Managed(0) && !m.vertices.is_empty() && !m.indices.is_empty() =>
                {
                    Some((c.clip_rect, m))
                }
                _ => None,
            })
            .collect();

        let ver_word = (self.atlas_version & 0x0FFF_FFFF) | ((cursor_code & 0x7) << 28) | if anim_active { 0x8000_0000 } else { 0 };
        let mut buf = Vec::with_capacity(4096);
        buf.extend_from_slice(b"MWGL");
        for u in [ver_word, self.atlas_width, self.atlas_height, meshes.len() as u32] {
            buf.extend_from_slice(&u.to_le_bytes());
        }
        for (clip, mesh) in meshes {
            for f in [clip.min.x, clip.min.y, clip.max.x, clip.max.y] {
                buf.extend_from_slice(&f.to_le_bytes());
            }
            buf.extend_from_slice(&(mesh.vertices.len() as u32).to_le_bytes());
            buf.extend_from_slice(&(mesh.indices.len() as u32).to_le_bytes());
            for v in &mesh.vertices {
                for f in [v.pos.x, v.pos.y, v.uv.x, v.uv.y] {
                    buf.extend_from_slice(&f.to_le_bytes());
                }
                buf.extend_from_slice(&v.color.to_array());
            }
            for &idx in &mesh.indices {
                buf.extend_from_slice(&idx.to_le_bytes());
            }
        }
        buf
    }

    fn apply_texture_deltas(&mut self, delta: &egui::TexturesDelta) {
        if let Some(deltas) = delta.set.get(&egui::TextureId::Managed(0)) {
            for img_delta in deltas {
                let egui::ImageData::Color(ref color_img) = img_delta.image;
                let [w, h] = color_img.size;
                if let Some([px, py]) = img_delta.pos {
                    let (dst_w, dst_h) = (self.atlas_width as usize, self.atlas_height as usize);
                    for (y, x) in (0..h).flat_map(|y| (0..w).map(move |x| (y, x))) {
                        let (dx, dy) = (px + x, py + y);
                        let dst_idx = (dy * dst_w + dx) * 4;
                        if dx < dst_w && dy < dst_h && dst_idx + 4 <= self.atlas_rgba.len() {
                            self.atlas_rgba[dst_idx..dst_idx + 4].copy_from_slice(&color_img.pixels[y * w + x].to_array());
                        }
                    }
                } else {
                    (self.atlas_width, self.atlas_height) = (w as u32, h as u32);
                    self.atlas_rgba = color_img.pixels.iter().flat_map(|p| p.to_array()).collect();
                }
                self.atlas_version = self.atlas_version.wrapping_add(1).max(1);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_lan_ip() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .ok()
        .and_then(|s| s.connect("8.8.8.8:80").ok().and_then(|_| s.local_addr().ok()))
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

/// Spawns the background Mobile & Tablet WebGL2 HTTP server on `0.0.0.0:8080` (or `8081`).
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_background_mobile_server() -> Option<String> {
    let (listener, port) = TcpListener::bind("0.0.0.0:8080")
        .map(|l| (l, 8080u16))
        .or_else(|_| TcpListener::bind("0.0.0.0:8081").map(|l| (l, 8081u16)))
        .ok()?;
    let url = format!("http://{}:{port}", detect_lan_ip());
    let _ = std::thread::Builder::new().name("mobile_webgl_srv".into()).stack_size(8 * 1024 * 1024).spawn(move || {
        let session = Arc::new(Mutex::new(MobileWebSession::new()));
        for stream in listener.incoming().flatten() {
            handle_http_conn(stream, &Arc::clone(&session));
        }
    });
    Some(url)
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_query_viewport(first_line: &str) -> (f32, f32, f32) {
    let (mut w, mut h, mut dpr) = (390.0f32, 844.0f32, 2.0f32);
    let Some(qs) = first_line.split_whitespace().nth(1).and_then(|p| p.split('?').nth(1)) else {
        return (w, h, dpr);
    };
    for pair in qs.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "w" => w = v.parse().unwrap_or(w),
                "h" => h = v.parse().unwrap_or(h),
                "dpr" => dpr = v.parse().unwrap_or(dpr),
                _ => {}
            }
        }
    }
    (w, h, dpr)
}

#[cfg(not(target_arch = "wasm32"))]
fn read_http_request(stream: &mut std::net::TcpStream) -> Option<String> {
    let (mut buf, mut chunk, mut header_end, mut content_len) = (Vec::with_capacity(4096), [0u8; 4096], None, 0usize);
    for _ in 0..16 {
        let n = stream.read(&mut chunk).ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if header_end.is_none() {
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                header_end = Some(pos + 4);
                for line in String::from_utf8_lossy(&buf[..pos]).lines() {
                    if let Some(rest) = line.strip_prefix("Content-Length:").or_else(|| line.strip_prefix("content-length:")) {
                        content_len = rest.trim().parse::<usize>().unwrap_or(0);
                    }
                }
            }
        }
        if header_end.is_some_and(|h_end| buf.len() >= h_end + content_len) {
            break;
        }
    }
    (!buf.is_empty()).then(|| String::from_utf8_lossy(&buf).into_owned())
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_http_conn(mut stream: std::net::TcpStream, session: &Arc<Mutex<MobileWebSession>>) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(500)));
    let Some(req_str) = read_http_request(&mut stream) else {
        return;
    };
    let first_line = req_str.lines().next().unwrap_or("");
    let body = req_str.split("\r\n\r\n").nth(1).unwrap_or("");
    let (qw, qh, qdpr) = parse_query_viewport(first_line);

    let write_ok = |st: &mut std::net::TcpStream, ct: &str, data: &[u8]| {
        let hdr = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            data.len()
        );
        let _ = st.write_all(hdr.as_bytes());
        let _ = st.write_all(data);
    };
    if first_line.starts_with("GET /atlas.bin") {
        let payload = session.lock().ok().map(|s| s.encode_atlas()).unwrap_or_default();
        write_ok(&mut stream, "application/octet-stream", &payload);
    } else if first_line.starts_with("POST /frame") {
        let payload = session.lock().ok().map(|mut s| s.encode_frame(qw, qh, qdpr, body)).unwrap_or_default();
        write_ok(&mut stream, "application/octet-stream", &payload);
    } else if first_line.starts_with("GET /macview_gallery.wasm") {
        if let Ok(wasm_bytes) = std::fs::read("macview_gallery.wasm") {
            write_ok(&mut stream, "application/wasm", &wasm_bytes);
        } else {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        }
    } else {
        write_ok(&mut stream, "text/html; charset=utf-8", MOBILE_WEB_HTML.as_bytes());
    }
}

impl GalleryApp {
    pub(crate) fn render_mobile_compact_work_area(&mut self, work_ui: &mut Ui, theme: &MacTheme) {
        let full_r = work_ui.max_rect();
        let bar_h = 34.0;
        let bar_rect = Rect::from_min_size(full_r.min, vec2(full_r.width(), bar_h));
        let pane_rect = Rect::from_min_max(pos2(full_r.min.x, full_r.min.y + bar_h + 2.0), full_r.max);

        let mut bar_ui = work_ui.new_child(
            egui::UiBuilder::new().id_salt("mobile_top_switcher_bar").max_rect(bar_rect).layout(Layout::left_to_right(Align::Center)),
        );
        bar_ui.add_space(6.0);
        let mut next_pane = self.mobile_pane_idx;
        MacSegmentedControl::new(&["Canvas", "Inspector", "Nav"], self.mobile_pane_idx)
            .theme(theme)
            .height(24.0)
            .show(&mut bar_ui, |idx| next_pane = idx);
        self.mobile_pane_idx = next_pane;
        bar_ui.add_space(6.0);
        for (i, scr) in GalleryScreen::ALL.iter().copied().enumerate() {
            let lbl = format!("S{}", i + 1);
            if bar_ui.add(MacButtonCell::new(&lbl).theme(theme).selected(self.active_screen == scr)).clicked() {
                self.active_screen = scr;
                self.sidebar.active_screen = scr;
                self.sidebar.selected_node = scr.title().to_string();
                self.mobile_pane_idx = 0;
            }
        }

        let mut pane_ui = work_ui.new_child(
            egui::UiBuilder::new()
                .id_salt(("mobile_active_pane", self.mobile_pane_idx))
                .max_rect(pane_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        match self.mobile_pane_idx {
            1 => {
                let (snippet_ctx, mut m) = (super::snippet::SnippetContext::from(&self.center), self.inspector_mode);
                self.inspector.render(&mut pane_ui, theme, &self.card, (&mut m, self.active_screen), snippet_ctx, &self.toasts);
                self.inspector_mode = m;
            }
            2 => {
                let (mut m, mut s) = (self.inspector_mode, self.active_screen);
                self.sidebar.render(&mut pane_ui, theme, &mut m, &mut s, &self.toasts);
                self.inspector_mode = m;
                if self.active_screen != s {
                    (self.active_screen, self.mobile_pane_idx) = (s, 0);
                }
            }
            _ => {
                let mut s = self.active_screen;
                let args = super::center_state::CenterRenderArgs {
                    card_cell: &self.card,
                    undo_manager: &self.undo_manager,
                    last_stable_card: &self.last_stable_card,
                    inspector_mode: self.inspector_mode,
                    snapping_enabled: self.inspector.snapping_enabled,
                    unit_segmented_idx: self.inspector.unit_segmented_idx,
                    vis_canvas: self.sidebar.vis_canvas,
                    lock_canvas: self.sidebar.lock_canvas,
                };
                self.center.render(&mut pane_ui, theme, args, &mut s, &self.toasts);
                self.active_screen = s;
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(unsafe_code)]
mod wasm_bridge {
    use super::MobileWebSession;
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
            let on_avkit = u32::from(rt.session.app.active_screen == super::super::types::GalleryScreen::AvKitMediaStudio);
            on_avkit | (u32::from(rt.session.app.center.media_playing) << 1) | (((rt.session.app.center.codec_preset as u32) & 0xF) << 4)
        })
        .unwrap_or(0)
    }
    #[no_mangle]
    pub extern "C" fn wasm_media_rect(idx: i32) -> f32 {
        with_rt(|rt| {
            let r = rt.session.app.center.last_video_viewport_rect;
            [
                r.min.x,
                r.min.y,
                r.width().max(0.0),
                r.height().max(0.0),
                rt.session.app.center.media_gain.clamp(0.0, 1.0),
                rt.session.app.center.media_speed.clamp(0.25, 2.0),
            ]
            .get(idx as usize)
            .copied()
            .unwrap_or(0.0)
        })
        .unwrap_or(0.0)
    }
}
