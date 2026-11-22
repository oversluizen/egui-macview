//! Tool- & App-Agnostic Multi-Texture `MWGL` WebGL2 / WASM Session (`MacWebGlSession`, `SPEC-016 §9.1`, `REQ-016-09`).
//!
//! Packs both the `egui` font/symbol atlas (`TextureId::Managed(0)`) AND all dynamically generated
//! 3-Matrix Liquid Glass `TextureHandle`s (`TextureId::Managed(1..N)`) into a single combined
//! RGBA8 WebGL2 Mega-Atlas, remapping mesh vertex `(u, v)` coordinates transparently so any
//! `egui-macview` host application renders identically in WebAssembly, Safari, and Chrome.

use crate::symbols::setup_macview_fonts;
use egui::{pos2, vec2, Context, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, TextureId, Ui};
use std::collections::BTreeMap;

#[derive(Clone)]
struct SubTextureEntry {
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

/// App-agnostic headless WebGL2/WASM session serializer for `egui-macview` (`REQ-016-09`).
///
/// # Example
/// ```rust
/// use egui_macview::MacWebGlSession;
///
/// let mut session = MacWebGlSession::new();
/// let frame = session.encode_frame(360.0, 240.0, 2.0, "", |ui| {
///     ui.label("Hello WebGL2");
/// });
/// assert!(frame.starts_with(b"MWGL"));
/// ```
pub struct MacWebGlSession {
    /// Underlying headless [`egui::Context`].
    pub ctx: Context,
    textures: BTreeMap<TextureId, SubTextureEntry>,
    /// Combined RGBA8 Mega-Atlas byte buffer.
    pub atlas_rgba: Vec<u8>,
    /// Combined Mega-Atlas width in pixels.
    pub atlas_width: u32,
    /// Combined Mega-Atlas height in pixels.
    pub atlas_height: u32,
    uv_rects: BTreeMap<TextureId, (u32, u32, u32, u32)>,
    /// Monotonic Mega-Atlas version counter incremented whenever any managed texture changes.
    pub atlas_version: u32,
    /// Optional dirty sub-rectangle `(x, y, w, h)` in the Mega-Atlas for partial `gl.texSubImage2D` uploads (`REQ-016-B1.12c`).
    pub atlas_dirty_rect: Option<(u32, u32, u32, u32)>,
    atlas_dirty: bool,
    /// Current session clock in seconds.
    pub time_secs: f64,
    /// Last pointer coordinate in logical points.
    pub last_pos: Pos2,
    /// Last viewport `(width_pt, height_pt, dpr)`.
    pub last_viewport: (f32, f32, f32),
    pending_custom_texts: Vec<String>,
}

impl Default for MacWebGlSession {
    fn default() -> Self {
        Self::new()
    }
}

impl MacWebGlSession {
    /// Creates a new app-agnostic `MacWebGlSession` with `Inter` and `CupertinoIcons` fonts installed.
    pub fn new() -> Self {
        let ctx = Context::default();
        let mut fonts = egui::FontDefinitions::default();
        setup_macview_fonts(&mut fonts);
        ctx.set_fonts(fonts);
        Self::with_context(ctx)
    }

    /// Creates a new `MacWebGlSession` wrapping an existing pre-configured [`egui::Context`].
    pub fn with_context(ctx: Context) -> Self {
        Self {
            ctx,
            textures: BTreeMap::new(),
            atlas_rgba: Vec::new(),
            atlas_width: 0,
            atlas_height: 0,
            uv_rects: BTreeMap::new(),
            atlas_version: 0,
            atlas_dirty_rect: None,
            atlas_dirty: false,
            time_secs: 1.0,
            last_pos: pos2(180.0, 200.0),
            last_viewport: (390.0, 844.0, 2.0),
            pending_custom_texts: Vec::new(),
        }
    }

    /// Returns a reference to the underlying [`egui::Context`].
    pub fn context(&self) -> &Context {
        &self.ctx
    }

    /// Returns the monotonic Mega-Atlas version counter incremented whenever any managed texture changes.
    pub fn atlas_version(&self) -> u32 {
        self.atlas_version
    }

    /// Returns the number of distinct `TextureId`s currently packed into the WebGL2 Mega-Atlas.
    pub fn packed_texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Takes the accumulated Mega-Atlas dirty rectangle `(x, y, width, height)` for `gl.texSubImage2D` (`REQ-016-B1.12c`).
    pub fn take_atlas_dirty_rect(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.atlas_dirty_rect.take()
    }

    /// Returns the maximum Y pixel coordinate occupied by `TextureId::Managed(0)` (the font/icon atlas).
    pub fn font_atlas_max_y(&self) -> u32 {
        self.uv_rects.get(&TextureId::Managed(0)).map(|&(_, oy, _, th)| oy + th).unwrap_or(0)
    }

    /// Drains any custom `\u{1f}`-prefixed payload lines received via `T \u{1f}...` during `encode_frame`.
    pub fn drain_custom_payloads(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_custom_texts)
    }

    /// Encodes the combined RGBA8 WebGL2 Mega-Atlas (`[width_u32_le, height_u32_le, rgba_bytes...]`).
    pub fn encode_atlas(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(8 + self.atlas_rgba.len());
        out.extend_from_slice(&self.atlas_width.to_le_bytes());
        out.extend_from_slice(&self.atlas_height.to_le_bytes());
        out.extend_from_slice(&self.atlas_rgba);
        out
    }

    /// Steps the UI across the batched input command log `body` and serializes the resulting
    /// multi-texture `MWGL` binary WebGL2 frame.
    pub fn encode_frame(&mut self, width_arg: f32, height_arg: f32, dpr_arg: f32, body: &str, mut render_ui: impl FnMut(&mut Ui)) -> Vec<u8> {
        if (width_arg - 390.0).abs() > 0.5 || (height_arg - 844.0).abs() > 0.5 {
            self.last_viewport = (width_arg.clamp(240.0, 2560.0), height_arg.clamp(240.0, 2560.0), dpr_arg.clamp(1.0, 3.0));
        }
        let (mut width, mut height, mut dpr) = self.last_viewport;
        let mut event_passes: Vec<Vec<Event>> = vec![Vec::new()];
        let mut has_press_in_current_pass = false;
        let mut client_clock: Option<f64> = None;

        for line in body.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.as_slice() {
                ["V", w, h, d] => {
                    width = w.parse::<f32>().unwrap_or(width).clamp(240.0, 2560.0);
                    height = h.parse::<f32>().unwrap_or(height).clamp(240.0, 2560.0);
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
                        cur.push(Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE });
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
                        cur.push(Event::PointerButton { pos: p, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE });
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
                    let payload = &line[2..];
                    if let Some(rest) = payload.strip_prefix('\u{1f}') {
                        self.pending_custom_texts.push(rest.to_string());
                    } else if let Some(cur) = event_passes.last_mut().filter(|_| !payload.is_empty()) {
                        cur.push(Event::Text(payload.to_string()));
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

            let mut full_output = self.ctx.run_ui(raw, |ui| {
                render_ui(ui);
            });
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
        if self.atlas_dirty {
            self.rebuild_mega_atlas();
        }
        let mw = (self.atlas_width.max(1)) as f32;
        let mh = (self.atlas_height.max(1)) as f32;

        let meshes: Vec<_> = self
            .ctx
            .tessellate(last_shapes, dpr)
            .into_iter()
            .filter_map(|c| match c.primitive {
                egui::epaint::Primitive::Mesh(mut m) if !m.vertices.is_empty() && !m.indices.is_empty() => {
                    let &(ox, oy, tw, th) = self.uv_rects.get(&m.texture_id)?;
                    let (ox_f, oy_f, tw_f, th_f) = (ox as f32, oy as f32, tw as f32, th as f32);
                    for v in &mut m.vertices {
                        v.uv.x = (ox_f + v.uv.x.clamp(0.0, 1.0) * tw_f) / mw;
                        v.uv.y = (oy_f + v.uv.y.clamp(0.0, 1.0) * th_f) / mh;
                    }
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
        let mut entries: Vec<_> = delta.set.iter().collect();
        entries.sort_by_key(|(id, _)| *id);
        for (&tex_id, deltas) in entries {
            for img_delta in deltas {
                let egui::ImageData::Color(ref color_img) = img_delta.image;
                let [w, h] = color_img.size;
                if let Some([px, py]) = img_delta.pos {
                    if let Some(sub) = self.textures.get_mut(&tex_id) {
                        for (y, x) in (0..h).flat_map(|y| (0..w).map(move |x| (y, x))) {
                            let (dx, dy) = (px + x, py + y);
                            let dst_idx = (dy * sub.width + dx) * 4;
                            if dx < sub.width && dy < sub.height && dst_idx + 4 <= sub.rgba.len() {
                                sub.rgba[dst_idx..dst_idx + 4].copy_from_slice(&color_img.pixels[y * w + x].to_array());
                            }
                        }
                        self.atlas_dirty = true;
                    }
                } else {
                    let rgba: Vec<u8> = color_img.pixels.iter().flat_map(|p| p.to_array()).collect();
                    let in_place = if let (Some(sub), Some(&(ox, oy, tw, th))) = (self.textures.get_mut(&tex_id), self.uv_rects.get(&tex_id)) {
                        if sub.width == w && sub.height == h && (tw as usize) == w && (th as usize) == h && !self.atlas_dirty {
                            sub.rgba.copy_from_slice(&rgba);
                            let mw = self.atlas_width as usize;
                            for row in 0..h {
                                let src_s = row * w * 4;
                                let dst_s = (((oy as usize) + row) * mw + (ox as usize)) * 4;
                                if dst_s + w * 4 <= self.atlas_rgba.len() {
                                    self.atlas_rgba[dst_s..dst_s + w * 4].copy_from_slice(&rgba[src_s..src_s + w * 4]);
                                }
                            }
                            self.atlas_version = self.atlas_version.wrapping_add(1).max(1);
                            self.atlas_dirty_rect = Some(match self.atlas_dirty_rect {
                                Some((rx, ry, rw, rh)) => {
                                    let (x0, y0) = (rx.min(ox), ry.min(oy));
                                    let (x1, y1) = ((rx + rw).max(ox + tw), (ry + rh).max(oy + th));
                                    (x0, y0, x1 - x0, y1 - y0)
                                }
                                None => (ox, oy, tw, th),
                            });
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !in_place {
                        self.textures.insert(tex_id, SubTextureEntry { width: w, height: h, rgba });
                        self.atlas_dirty = true;
                    }
                }
            }
        }
        if !delta.free.is_empty() {
            let mut freed: Vec<_> = delta.free.iter().copied().collect();
            freed.sort();
            for tex_id in freed {
                if self.textures.remove(&tex_id).is_some() {
                    self.atlas_dirty = true;
                }
            }
        }
    }

    fn rebuild_mega_atlas(&mut self) {
        self.uv_rects.clear();
        let font_w = self.textures.get(&TextureId::Managed(0)).map(|t| t.width as u32).unwrap_or(1024);
        let max_sub_w = self.textures.values().map(|t| t.width as u32).max().unwrap_or(512);
        let mega_w = font_w.max(max_sub_w).max(512);
        let pad = 2u32;
        let (mut cur_x, mut cur_y, mut row_h) = (0u32, 0u32, 0u32);

        for (&tex_id, sub) in &self.textures {
            let (tw, th) = (sub.width as u32, sub.height as u32);
            if tex_id == TextureId::Managed(0) {
                self.uv_rects.insert(tex_id, (0, 0, tw, th));
                cur_y = th + pad;
            } else {
                if cur_x > 0 && cur_x + tw + pad > mega_w {
                    cur_x = 0;
                    cur_y += row_h + pad;
                    row_h = 0;
                }
                self.uv_rects.insert(tex_id, (cur_x, cur_y, tw, th));
                cur_x += tw + pad;
                row_h = row_h.max(th);
            }
        }
        let mega_h = (cur_y + row_h + pad).max(1);
        let mut buf = vec![0u8; (mega_w as usize) * (mega_h as usize) * 4];
        for (&tex_id, sub) in &self.textures {
            if let Some(&(ox, oy, tw, th)) = self.uv_rects.get(&tex_id) {
                for row in 0..(th as usize) {
                    let src_start = row * (tw as usize) * 4;
                    let src_end = src_start + (tw as usize) * 4;
                    let dst_start = (((oy as usize) + row) * (mega_w as usize) + (ox as usize)) * 4;
                    let dst_end = dst_start + (tw as usize) * 4;
                    if src_end <= sub.rgba.len() && dst_end <= buf.len() {
                        buf[dst_start..dst_end].copy_from_slice(&sub.rgba[src_start..src_end]);
                    }
                }
            }
        }
        self.atlas_width = mega_w;
        self.atlas_height = mega_h;
        self.atlas_rgba = buf;
        self.atlas_dirty = false;
        self.atlas_dirty_rect = Some((0, 0, mega_w, mega_h));
        self.atlas_version = self.atlas_version.wrapping_add(1).max(1);
    }
}
