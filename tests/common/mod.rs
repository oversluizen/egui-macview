//! Canonical COTS Test Harness for `egui-macview` (`REQ-001-06.3`).
//!
//! Provides deterministic headless `egui::Context` initialization, multi-frame pointer click/drag
//! simulation, `MacSceneIR` widget rect lookup by `ib_object_id`, and shape inspection helpers.

#![allow(dead_code)]

use egui::{epaint::TextShape, Color32, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Shape, Vec2};
use egui_macview::{
    scene::{LayoutPolicy, MacSceneIR},
    setup_symbols_font, MacAppearance, MacTheme,
};

/// Creates a deterministic headless [`egui::Context`] with open-source fonts and [`MacTheme`] installed.
pub fn headless_ctx(appearance: MacAppearance) -> (Context, MacTheme) {
    let ctx = Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);
    let theme = MacTheme::new(appearance);
    theme.apply_to_egui(&ctx);
    (ctx, theme)
}

/// Runs a single headless UI frame at `viewport` size and returns the clipped shapes.
pub fn run_frame(ctx: &Context, mut input: RawInput, viewport: Vec2, mut f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    input.screen_rect = Some(Rect::from_min_size(Pos2::ZERO, viewport));
    let mut out = ctx.run_ui(input, |ui| {
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, &mut f);
    });
    out.textures_delta.clear();
    out.shapes
}

/// Executes a genuine 3-frame `egui` primary click (`Hover -> Press -> Release`) at `pos`.
pub fn click_at_multi_frame(ctx: &Context, pos: Pos2, viewport: Vec2, mut f: impl FnMut(&mut egui::Ui)) -> Vec<egui::epaint::ClippedShape> {
    let hover = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![Event::PointerMoved(pos)],
        ..Default::default()
    };
    let _ = run_frame(ctx, hover, viewport, &mut f);

    let press = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    let _ = run_frame(ctx, press, viewport, &mut f);

    let release = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        ],
        ..Default::default()
    };
    run_frame(ctx, release, viewport, &mut f)
}

/// Resolves the solved `egui::Rect` of a node in `scene` by its `ib_object_id` (`REQ-001-06.3`).
pub fn resolved_rect_by_id(scene: &MacSceneIR, ib_object_id: &str, viewport: Vec2, policy: LayoutPolicy) -> Option<Rect> {
    let node = scene.nodes().iter().find(|n| n.ib_object_id == ib_object_id)?;
    let solved = scene.solve_layout(viewport, policy, 0, "");
    solved.get(&node.id).copied()
}

/// Recursively collects all [`TextShape`] primitives from a [`Shape`] tree.
pub fn collect_text_shapes<'a>(shape: &'a Shape, out: &mut Vec<&'a TextShape>) {
    match shape {
        Shape::Text(t) => out.push(t),
        Shape::Vec(v) => {
            for s in v {
                collect_text_shapes(s, out);
            }
        }
        _ => {}
    }
}

/// Finds the stroke color of the first `Shape::LineSegment` whose midpoint lies inside `target_rect`.
pub fn first_line_stroke_color_in_rect(shapes: &[egui::epaint::ClippedShape], target_rect: Rect) -> Option<Color32> {
    fn search_shape(shape: &Shape, target_rect: Rect) -> Option<Color32> {
        match shape {
            Shape::LineSegment { points, stroke } => {
                let mid = Pos2::new((points[0].x + points[1].x) * 0.5, (points[0].y + points[1].y) * 0.5);
                target_rect.expand(2.0).contains(mid).then_some(stroke.color)
            }
            Shape::Vec(list) => list.iter().find_map(|s| search_shape(s, target_rect)),
            _ => None,
        }
    }
    shapes.iter().find_map(|cs| search_shape(&cs.shape, target_rect))
}

/// Deterministic software `2x` Retina RGBA8 framebuffer rasterized directly from `egui::Context::tessellate`.
pub struct RetinaCpuFramebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>,
}

impl RetinaCpuFramebuffer {
    pub fn render_retina(ctx: &Context, viewport: Vec2, scale: f32, bg: Color32, mut ui_fn: impl FnMut(&mut egui::Ui)) -> Self {
        let mut fonts_map: std::collections::BTreeMap<egui::TextureId, (usize, usize, Vec<[u8; 4]>)> = std::collections::BTreeMap::new();
        let mut shapes_out = Vec::new();
        for _ in 0..2 {
            let evs = ctx.data(|d| d.get_temp::<Vec<Event>>(egui::Id::NULL.with("retina_events"))).unwrap_or_default();
            let input = RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)),
                events: evs,
                viewports: std::iter::once((
                    egui::ViewportId::ROOT,
                    egui::ViewportInfo { native_pixels_per_point: Some(scale), ..Default::default() },
                ))
                .collect(),
                ..Default::default()
            };
            let mut full = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, &mut ui_fn);
            });
            shapes_out = full.shapes;
            let _ = ctx.tessellate(shapes_out.clone(), scale);
            let mut tex_entries: Vec<_> = full.textures_delta.set.iter().collect();
            tex_entries.sort_by_key(|(id, _)| *id);
            for (&id, deltas) in tex_entries {
                for delta in deltas {
                    let [w, h] = delta.image.size();
                    let rgba: Vec<[u8; 4]> = match &delta.image {
                        egui::ImageData::Color(c) => c.pixels.iter().map(|p| p.to_array()).collect(),
                    };
                    if let Some([ox, oy]) = delta.pos {
                        if let Some((tw, _, buf)) = fonts_map.get_mut(&id) {
                            for row in 0..h {
                                for col in 0..w {
                                    buf[(oy + row) * *tw + (ox + col)] = rgba[row * w + col];
                                }
                            }
                        }
                    } else {
                        fonts_map.insert(id, (w, h, rgba));
                    }
                }
            }
            full.textures_delta.clear();
        }
        ctx.fonts(|f| {
            let img = f.image();
            let [w, h] = img.size;
            let rgba: Vec<[u8; 4]> = img.pixels.iter().map(|p| p.to_array()).collect();
            fonts_map.insert(egui::TextureId::default(), (w, h, rgba));
        });

        let w_px = (viewport.x * scale).round() as usize;
        let h_px = (viewport.y * scale).round() as usize;
        let mut fb = Self { width: w_px, height: h_px, pixels: vec![bg.to_array(); w_px * h_px] };
        let primitives = ctx.tessellate(shapes_out, scale);
        for prim in primitives {
            let clip = Rect::from_min_max(prim.clip_rect.min * scale, prim.clip_rect.max * scale);
            let egui::epaint::Primitive::Mesh(mesh) = prim.primitive else { continue };
            let tex = fonts_map.get(&mesh.texture_id);
            for &[i0, i1, i2] in mesh.indices.as_chunks::<3>().0 {
                let (mut v0, mut v1, mut v2) = (mesh.vertices[i0 as usize], mesh.vertices[i1 as usize], mesh.vertices[i2 as usize]);
                v0.pos *= scale;
                v1.pos *= scale;
                v2.pos *= scale;
                fb.rasterize_triangle(v0, v1, v2, clip, tex.map(|(w, h, b)| (*w, *h, b.as_slice())));
            }
        }
        fb
    }

    fn rasterize_triangle(
        &mut self,
        v0: egui::epaint::Vertex,
        v1: egui::epaint::Vertex,
        v2: egui::epaint::Vertex,
        clip: Rect,
        tex: Option<(usize, usize, &[[u8; 4]])>,
    ) {
        let min_x = v0.pos.x.min(v1.pos.x).min(v2.pos.x).max(clip.min.x).floor().max(0.0) as usize;
        let max_x = (v0.pos.x.max(v1.pos.x).max(v2.pos.x).min(clip.max.x).ceil() as usize).min(self.width);
        let min_y = v0.pos.y.min(v1.pos.y).min(v2.pos.y).max(clip.min.y).floor().max(0.0) as usize;
        let max_y = (v0.pos.y.max(v1.pos.y).max(v2.pos.y).min(clip.max.y).ceil() as usize).min(self.height);
        let area = (v1.pos.x - v0.pos.x) * (v2.pos.y - v0.pos.y) - (v1.pos.y - v0.pos.y) * (v2.pos.x - v0.pos.x);
        if area.abs() < 1e-5 {
            return;
        }
        let inv_area = 1.0 / area;
        for py in min_y..max_y {
            for px in min_x..max_x {
                let p = Pos2::new(px as f32 + 0.5, py as f32 + 0.5);
                if !clip.contains(p) {
                    continue;
                }
                let w0 = ((v1.pos.x - p.x) * (v2.pos.y - p.y) - (v1.pos.y - p.y) * (v2.pos.x - p.x)) * inv_area;
                let w1 = ((v2.pos.x - p.x) * (v0.pos.y - p.y) - (v2.pos.y - p.y) * (v0.pos.x - p.x)) * inv_area;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                    continue;
                }
                let (c0, c1, c2) = (v0.color.to_array(), v1.color.to_array(), v2.color.to_array());
                let mut rgba = [0u8; 4];
                for ch in 0..4 {
                    rgba[ch] = (w0 * c0[ch] as f32 + w1 * c1[ch] as f32 + w2 * c2[ch] as f32).round().clamp(0.0, 255.0) as u8;
                }
                if let Some((tw, th, tbuf)) = tex {
                    let u = (w0 * v0.uv.x + w1 * v1.uv.x + w2 * v2.uv.x).clamp(0.0, 1.0);
                    let v = (w0 * v0.uv.y + w1 * v1.uv.y + w2 * v2.uv.y).clamp(0.0, 1.0);
                    if u > 0.0 || v > 0.0 {
                        let fx = (u * (tw as f32) - 0.5).clamp(0.0, (tw.saturating_sub(1)) as f32);
                        let fy = (v * (th as f32) - 0.5).clamp(0.0, (th.saturating_sub(1)) as f32);
                        let x0 = fx.floor() as usize;
                        let y0 = fy.floor() as usize;
                        let x1 = (x0 + 1).min(tw.saturating_sub(1));
                        let y1 = (y0 + 1).min(th.saturating_sub(1));
                        let tx = fx - x0 as f32;
                        let ty = fy - y0 as f32;
                        let p00 = tbuf[y0 * tw + x0];
                        let p10 = tbuf[y0 * tw + x1];
                        let p01 = tbuf[y1 * tw + x0];
                        let p11 = tbuf[y1 * tw + x1];
                        for ch in 0..4 {
                            let top = p00[ch] as f32 * (1.0 - tx) + p10[ch] as f32 * tx;
                            let bot = p01[ch] as f32 * (1.0 - tx) + p11[ch] as f32 * tx;
                            let tcol = (top * (1.0 - ty) + bot * ty).round().clamp(0.0, 255.0) as u16;
                            rgba[ch] = ((rgba[ch] as u16 * tcol) / 255) as u8;
                        }
                    }
                }
                let a = rgba[3] as u16;
                if a > 0 {
                    let dst = &mut self.pixels[py * self.width + px];
                    let inv_a = 255u16.saturating_sub(a);
                    for ch in 0..3 {
                        dst[ch] = ((rgba[ch] as u16 + (dst[ch] as u16 * inv_a) / 255).min(255)) as u8;
                    }
                    dst[3] = 255;
                }
            }
        }
    }

    pub fn save_png(&self, path: &std::path::Path) {
        let mut raw = Vec::with_capacity(self.height * (1 + self.width * 4));
        for y in 0..self.height {
            raw.push(0u8);
            for x in 0..self.width {
                raw.extend_from_slice(&self.pixels[y * self.width + x]);
            }
        }
        let mut zlib = vec![0x78, 0x01];
        for (i, chunk) in raw.chunks(65535).enumerate() {
            let last = (i + 1) * 65535 >= raw.len();
            zlib.push(u8::from(last));
            let len = chunk.len() as u16;
            zlib.extend_from_slice(&len.to_le_bytes());
            zlib.extend_from_slice(&(!len).to_le_bytes());
            zlib.extend_from_slice(chunk);
        }
        let (mut s1, mut s2) = (1u32, 0u32);
        for &b in &raw {
            s1 = (s1 + b as u32) % 65521;
            s2 = (s2 + s1) % 65521;
        }
        zlib.extend_from_slice(&((s2 << 16) | s1).to_be_bytes());
        fn push_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            out.extend_from_slice(tag);
            out.extend_from_slice(data);
            let mut crc = 0xFFFF_FFFFu32;
            for &b in tag.iter().chain(data.iter()) {
                crc ^= b as u32;
                for _ in 0..8 {
                    crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
                }
            }
            out.extend_from_slice(&(!crc).to_be_bytes());
        }
        let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(self.width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.height as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        push_chunk(&mut png, b"IHDR", &ihdr);
        push_chunk(&mut png, b"IDAT", &zlib);
        push_chunk(&mut png, b"IEND", &[]);
        let _ = std::fs::write(path, png);
    }
}
