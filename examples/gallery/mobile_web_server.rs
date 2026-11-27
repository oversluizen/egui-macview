//! Built-in Zero-Dependency Mobile & Tablet WebGL2 Browser Server (`REQ-013-02O`).
//!
//! Streams `egui::Context::tessellate` triangle meshes (`MWGL` binary protocol) and the
//! RGBA8 font/symbol atlas over HTTP (`0.0.0.0:8080`) so any iPhone, iPad, or Android
//! tablet on the local Wi-Fi network can interact with `macview_gallery` in Safari or Chrome.

use super::setup_system_or_custom_fonts;
use crate::{GalleryApp, GalleryScreen};
use egui::{pos2, vec2, Align, Context, Layout, Rect, Ui};
use egui_macview::{theme::MacTheme, MacButtonCell, MacSegmentedControl, MacWebGlSession};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    io::{Read, Write},
    net::{TcpListener, UdpSocket},
    sync::{Arc, Mutex},
};

#[cfg(not(target_arch = "wasm32"))]
const MOBILE_WEB_HTML: &str = include_str!("mobile_web_html.html");

/// Interactive headless `GalleryApp` session driving the mobile/tablet WebGL2 `<canvas>` via [`MacWebGlSession`].
pub(crate) struct MobileWebSession {
    pub(crate) web: MacWebGlSession,
    pub(crate) app: GalleryApp,
}

impl std::ops::Deref for MobileWebSession {
    type Target = MacWebGlSession;
    fn deref(&self) -> &Self::Target {
        &self.web
    }
}

impl std::ops::DerefMut for MobileWebSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.web
    }
}

impl MobileWebSession {
    pub(crate) fn new() -> Self {
        let ctx = Context::default();
        setup_system_or_custom_fonts(&ctx);
        #[allow(unused_mut)]
        let mut session = Self { web: MacWebGlSession::with_context(ctx), app: GalleryApp::new() };
        #[cfg(not(target_arch = "wasm32"))]
        let _ = session.encode_frame(390.0, 844.0, 2.0, "");
        session
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn encode_atlas(&self) -> Vec<u8> {
        self.web.encode_atlas()
    }

    pub(crate) fn encode_frame(&mut self, width_arg: f32, height_arg: f32, dpr_arg: f32, body: &str) -> Vec<u8> {
        let app = &mut self.app;
        let buf = self.web.encode_frame(width_arg, height_arg, dpr_arg, body, |ui| {
            app.render_ui(ui);
        });
        for rest in self.web.drain_custom_payloads() {
            let (name, kb) = rest.split_once('|').unwrap_or((&rest, "1024"));
            let ext = std::path::Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or("FILE").to_uppercase();
            self.app.center.finder_custom_entries.push(super::FinderCustomEntry {
                name: name.to_string(),
                path: std::path::PathBuf::from(format!("/Users/studio/Local_Disk/{name}")),
                is_dir: false,
                size_mb: (kb.parse::<u32>().unwrap_or(1024) / 1024).max(1),
                codec: ext,
                duration: "Local Disk".to_string(),
                rating: 5,
                tags: vec!["Local".to_string(), "Browser".to_string()],
            });
            self.app.center.finder_current_dir = Some(std::path::PathBuf::from("/Users/studio/Local_Disk"));
        }
        buf
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
        let (full_r, bar_h) = (work_ui.max_rect(), 34.0);
        let bar_rect = Rect::from_min_size(full_r.min, vec2(full_r.width(), bar_h));
        let pane_rect = Rect::from_min_max(pos2(full_r.min.x, full_r.min.y + bar_h + 2.0), full_r.max);
        let mut bar_ui = work_ui.new_child(
            egui::UiBuilder::new().id_salt("mobile_top_switcher_bar").max_rect(bar_rect).layout(Layout::left_to_right(Align::Center)),
        );
        let mut next_pane = self.mobile_pane_idx;
        MacSegmentedControl::new(&["Canvas", "Inspector", "Nav"], self.mobile_pane_idx)
            .theme(theme)
            .height(24.0)
            .show(&mut bar_ui, |idx| next_pane = idx);
        self.mobile_pane_idx = next_pane;
        for (i, scr) in GalleryScreen::ALL.iter().copied().enumerate() {
            if bar_ui.add(MacButtonCell::new(&format!("S{}", i + 1)).theme(theme).selected(self.active_screen == scr)).clicked() {
                (self.active_screen, self.sidebar.active_screen, self.sidebar.selected_node, self.mobile_pane_idx) =
                    (scr, scr, scr.title().to_string(), 0);
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
