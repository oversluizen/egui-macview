//! Normative COTS Tests for 1:1 Xcode `.nib` Coordinate Bijection (`REQ-013-08.1`),
//! AppKit `-[NSTextFieldCell drawingRectForBounds:]` `2.0pt` Inset & Zero Truncation (`REQ-013-08.2`),
//! and `MacThumbnailCell` `clipsToBounds = YES` Strict Cell & Footer Isolation (`REQ-013-08.3`).

mod common;

use egui::{pos2, vec2, Color32, Context, Pos2, RawInput, Rect, Vec2};
use egui_macview::{
    controls::MacControlSize,
    scene::{
        BindingId, CompiledConstraintSchedule, LayoutPolicy, MacSceneDataSource, MacSceneIR, NodeId, SceneNode, SceneNodeKind,
        ValueTransformer, VisibilityRule,
    },
    setup_custom_macview_fonts, setup_symbols_font, MacAppearance, MacInputCell, MacTheme, MacThumbnailCell, INTER_REGULAR_FONT_BYTES,
    INTER_SEMIBOLD_FONT_BYTES,
};
use std::collections::BTreeMap;

/// Deterministic software RGBA8 framebuffer rasterized directly from `egui::Context::tessellate`.
struct CpuFramebuffer {
    width: usize,
    height: usize,
    pixels: Vec<[u8; 4]>,
}

impl CpuFramebuffer {
    fn render(ctx: &Context, viewport: Vec2, bg: Color32, mut ui_fn: impl FnMut(&mut egui::Ui)) -> (Self, Vec<egui::epaint::ClippedShape>) {
        let mut fonts_map: BTreeMap<egui::TextureId, (usize, usize, Vec<[u8; 4]>)> = BTreeMap::new();
        let mut shapes_out = Vec::new();
        for pass in 0..2 {
            let input = RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, viewport)), ..Default::default() };
            let mut full = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, &mut ui_fn);
            });
            shapes_out = full.shapes;
            let _ = ctx.tessellate(shapes_out.clone(), 1.0);
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
            let _ = pass;
        }
        ctx.fonts(|f| {
            let img = f.image();
            let [w, h] = img.size;
            let rgba: Vec<[u8; 4]> = img.pixels.iter().map(|p| p.to_array()).collect();
            fonts_map.insert(egui::TextureId::default(), (w, h, rgba));
        });

        let w_px = viewport.x.round() as usize;
        let h_px = viewport.y.round() as usize;
        let mut fb = Self { width: w_px, height: h_px, pixels: vec![bg.to_array(); w_px * h_px] };
        let primitives = ctx.tessellate(shapes_out.clone(), 1.0);
        for prim in primitives {
            let clip = prim.clip_rect;
            let egui::epaint::Primitive::Mesh(mesh) = prim.primitive else { continue };
            let tex = fonts_map.get(&mesh.texture_id);
            for &[i0, i1, i2] in mesh.indices.as_chunks::<3>().0 {
                let (v0, v1, v2) = (mesh.vertices[i0 as usize], mesh.vertices[i1 as usize], mesh.vertices[i2 as usize]);
                fb.rasterize_triangle(v0, v1, v2, clip, tex.map(|(w, h, b)| (*w, *h, b.as_slice())));
            }
        }
        (fb, shapes_out)
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
                let p = pos2(px as f32 + 0.5, py as f32 + 0.5);
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
                    rgba[ch] = (w0 * c0[ch] as f32 + w1 * c1[ch] as f32 + w2 * c2[ch] as f32).clamp(0.0, 255.0) as u8;
                }
                if let Some((tw, th, tbuf)) = tex {
                    let u = (w0 * v0.uv.x + w1 * v1.uv.x + w2 * v2.uv.x).clamp(0.0, 1.0);
                    let v = (w0 * v0.uv.y + w1 * v1.uv.y + w2 * v2.uv.y).clamp(0.0, 1.0);
                    if u > 0.0 || v > 0.0 {
                        let tx = ((u * tw as f32) as usize).min(tw.saturating_sub(1));
                        let ty = ((v * th as f32) as usize).min(th.saturating_sub(1));
                        let tcol = tbuf[ty * tw + tx];
                        for ch in 0..4 {
                            rgba[ch] = ((rgba[ch] as u16 * tcol[ch] as u16) / 255) as u8;
                        }
                    }
                }
                let a = rgba[3] as u16;
                if a > 0 {
                    let dst = &mut self.pixels[py * self.width + px];
                    for ch in 0..3 {
                        dst[ch] = ((rgba[ch] as u16 * a + dst[ch] as u16 * (255 - a)) / 255) as u8;
                    }
                    dst[3] = 255;
                }
            }
        }
    }

    fn count_differing_pixels(&self, rect: Rect, baseline: [u8; 4], tolerance: u8) -> usize {
        let x0 = (rect.min.x.floor().max(0.0) as usize).min(self.width);
        let x1 = (rect.max.x.ceil().max(0.0) as usize).min(self.width);
        let y0 = (rect.min.y.floor().max(0.0) as usize).min(self.height);
        let y1 = (rect.max.y.ceil().max(0.0) as usize).min(self.height);
        let mut count = 0;
        for y in y0..y1 {
            for x in x0..x1 {
                let p = self.pixels[y * self.width + x];
                if p[0].abs_diff(baseline[0]) > tolerance
                    || p[1].abs_diff(baseline[1]) > tolerance
                    || p[2].abs_diff(baseline[2]) > tolerance
                {
                    count += 1;
                }
            }
        }
        count
    }
}

struct ShapeBoundsStore {
    vals: BTreeMap<BindingId, String>,
}

impl MacSceneDataSource for ShapeBoundsStore {
    fn get_bool(&self, _id: BindingId) -> bool {
        true
    }
    fn set_bool(&mut self, _id: BindingId, _val: bool) {}
    fn get_f64(&self, _id: BindingId) -> f64 {
        0.0
    }
    fn set_f64(&mut self, _id: BindingId, _val: f64) {}
    fn get_i64(&self, _id: BindingId) -> i64 {
        4
    }
    fn set_i64(&mut self, _id: BindingId, _val: i64) {}
    fn get_str(&self, id: BindingId) -> &str {
        self.vals.get(&id).map(String::as_str).unwrap_or("100")
    }
    fn set_str(&mut self, id: BindingId, val: &str) {
        self.vals.insert(id, val.to_owned());
    }
    fn get_options(&self, _id: BindingId) -> &[String] {
        &[]
    }
    fn on_action(&mut self, _action: egui_macview::scene::ActionId, _sender: NodeId) {}
    fn render_custom_slot(&mut self, _ui: &mut egui::Ui, _theme: &MacTheme, _slot_class: &str, _rect: Rect) {}
}

fn build_inspector_view_gs_shape_bounds_scene() -> MacSceneIR {
    let mk = |id: u32, ib_id: &str, kind: SceneNodeKind, title: &str, frame: [f32; 4], bid: Option<u32>| SceneNode {
        id: NodeId(id),
        ib_object_id: ib_id.into(),
        parent: Some(NodeId(1)),
        children: vec![],
        kind,
        title: title.into(),
        tooltip: String::new(),
        control_size: "small".into(),
        nib_frame: frame,
        alignment_rect: frame,
        autoresizing_mask: 36, // flexibleMaxX | flexibleMinY
        enabled: true,
        value_binding: bid.map(BindingId),
        binding_path: bid.map(|b| format!("field_{b}")),
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name: None,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    };

    let root = SceneNode {
        id: NodeId(1),
        ib_object_id: "root-shape-bounds".into(),
        parent: None,
        children: (2..=10).map(NodeId).collect(),
        kind: SceneNodeKind::Container { is_flipped: false },
        title: "InspectorViewGSShapeBounds".into(),
        tooltip: String::new(),
        control_size: "regular".into(),
        nib_frame: [0.0, 0.0, 192.0, 44.0],
        alignment_rect: [0.0, 0.0, 192.0, 44.0],
        autoresizing_mask: 0,
        enabled: true,
        value_binding: None,
        binding_path: None,
        value_transformer: ValueTransformer::Identity,
        action: None,
        action_selector: None,
        icon_name: None,
        visibility: VisibilityRule::Always,
        labeled_by: None,
    };

    let nodes = vec![
        root,
        mk(2, "origin-btn", SceneNodeKind::PushButton { is_default: false, is_cancel: false }, "⊙", [4.0, 3.0, 37.0, 37.0], None),
        mk(3, "x-lbl", SceneNodeKind::Label, "X", [42.0, 23.0, 13.0, 16.0], None),
        mk(4, "x-fld", SceneNodeKind::TextField, "120", [55.0, 23.0, 54.0, 16.0], Some(1)),
        mk(5, "y-lbl", SceneNodeKind::Label, "Y", [42.0, 3.0, 13.0, 16.0], None),
        mk(6, "y-fld", SceneNodeKind::TextField, "-40", [55.0, 3.0, 54.0, 16.0], Some(2)),
        mk(7, "w-lbl", SceneNodeKind::Label, "W", [118.0, 23.0, 14.0, 16.0], None),
        mk(8, "w-fld", SceneNodeKind::TextField, "680", [133.0, 23.0, 54.0, 16.0], Some(3)),
        mk(9, "h-lbl", SceneNodeKind::Label, "H", [118.0, 3.0, 14.0, 16.0], None),
        mk(10, "h-fld", SceneNodeKind::TextField, "700", [133.0, 3.0, 54.0, 16.0], Some(4)),
    ];
    MacSceneIR::from_nodes(NodeId(1), [192.0, 44.0], nodes, CompiledConstraintSchedule::default())
}

/// Oracle 1 (`REQ-013-08.1`): 1:1 Coordinate Bijection (`<= 0.5 pt`) on `InspectorViewGSShapeBounds.nib` (`192 × 44 pt`).
#[test]
fn oracle_1_exact_apple_nib_coordinate_bijection_inspector_shape_bounds() {
    let scene = build_inspector_view_gs_shape_bounds_scene();
    let solved = scene.solve_exact_nib_layout();

    let expected_appkit_frames: &[(u32, &str, [f32; 4])] = &[
        (2, "origin-btn", [4.0, 3.0, 37.0, 37.0]),
        (3, "x-lbl", [42.0, 23.0, 13.0, 16.0]),
        (4, "x-fld", [55.0, 23.0, 54.0, 16.0]),
        (5, "y-lbl", [42.0, 3.0, 13.0, 16.0]),
        (6, "y-fld", [55.0, 3.0, 54.0, 16.0]),
        (7, "w-lbl", [118.0, 23.0, 14.0, 16.0]),
        (8, "w-fld", [133.0, 23.0, 54.0, 16.0]),
        (9, "h-lbl", [118.0, 3.0, 14.0, 16.0]),
        (10, "h-fld", [133.0, 3.0, 54.0, 16.0]),
    ];

    let design_h = 44.0f32;
    for &(id, name, [x_nib, y_nib, w, h]) in expected_appkit_frames {
        let r = solved[&NodeId(id)];
        let y_top = design_h - y_nib - h;
        assert!(
            (r.min.x - x_nib).abs() <= 0.5
                && (r.min.y - y_top).abs() <= 0.5
                && (r.width() - w).abs() <= 0.5
                && (r.height() - h).abs() <= 0.5,
            "Oracle 1 violation for {name}: expected top-left [{x_nib}, {y_top}, {w}, {h}], got {:?}",
            r
        );
    }

    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let mut store = ShapeBoundsStore {
        vals: BTreeMap::from([(BindingId(1), "120".into()), (BindingId(2), "-40".into()), (BindingId(3), "680".into()), (BindingId(4), "700".into())]),
    };
    let mut used_rect = Rect::NOTHING;
    let shapes = common::run_frame(&ctx, RawInput::default(), vec2(800.0, 600.0), |ui| {
        ui.scope(|s_ui| {
            scene.show(s_ui, &theme, &mut store, LayoutPolicy::ExactAppleNib, u64::MAX, "");
            used_rect = s_ui.min_rect();
        });
    });
    assert!((used_rect.width() - 192.0).abs() <= 0.5 && (used_rect.height() - 44.0).abs() <= 0.5);
    for cs in &shapes {
        if let egui::Shape::Rect(r) = &cs.shape {
            if r.fill == theme.control_background() {
                assert!(r.rect.max.x <= 192.5 && r.rect.max.y <= 44.5);
            }
        }
    }

    let real_nib_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../nib/crates/nib-gui/assets/nibs/Frameworks/App.framework/Versions/A/Resources/Base.lproj/InspectorViewShapeBounds.nib");
    if real_nib_path.exists() {
        let mapper = egui_macview::scene::CustomClassMapper::default();
        let real_scene = egui_macview::scene::NibArchiveFrontend::from_path(&real_nib_path, &mapper).expect("decode real nib");
        let root_r = real_scene.solve_exact_nib_layout()[&real_scene.root_id()];
        assert!(root_r.width() < 260.0 && root_r.height() < 100.0);
    }
}

/// Oracle 2 (`REQ-013-08.2`): AppKit `-[NSTextFieldCell drawingRectForBounds:]` (`2.0pt` inset) & Zero Text Truncation in `35×16`, `42×16`, and `32×16` NIB frames (tested both with built-in open-source `Inter` AND injected `setup_custom_macview_fonts`).
#[test]
fn oracle_2_mac_input_cell_nib_2pt_inset_zero_truncation_and_framebuffer_pixels() {
    for use_custom_font_injection in [false, true] {
        let ctx = Context::default();
        let mut fonts = egui::FontDefinitions::default();
        if use_custom_font_injection {
            setup_custom_macview_fonts(&mut fonts, INTER_REGULAR_FONT_BYTES.to_vec(), Some(INTER_SEMIBOLD_FONT_BYTES.to_vec()));
        } else {
            setup_symbols_font(&mut fonts);
        }
        ctx.set_fonts(fonts);
        let theme = MacTheme::new(MacAppearance::DarkAqua);
        theme.apply_to_egui(&ctx);

        let mut u_hex_35 = String::from("0048");
        let mut u_hex_42 = String::from("0048");
        let mut dim_100 = String::from("100");
        let mut dim_680 = String::from("680");
        let mut dim_92 = String::from("92");
        let mut u_hex_child_36x18 = String::from("0048");

        let r_35 = Rect::from_min_size(pos2(8.0, 8.0), vec2(35.0, 16.0));
        let r_42 = Rect::from_min_size(pos2(52.0, 8.0), vec2(42.0, 16.0));
        let r_32_a = Rect::from_min_size(pos2(104.0, 8.0), vec2(32.0, 16.0));
        let r_32_b = Rect::from_min_size(pos2(144.0, 8.0), vec2(32.0, 16.0));
        let r_32_c = Rect::from_min_size(pos2(184.0, 8.0), vec2(32.0, 16.0));
        let r_child_36x18 = Rect::from_min_size(pos2(224.0, 8.0), vec2(36.0, 18.0));

        let (fb, shapes) = CpuFramebuffer::render(&ctx, vec2(272.0, 34.0), theme.window_background(), |ui| {
            MacInputCell::new("hex_35", &mut u_hex_35)
                .theme(&theme)
                .control_size(MacControlSize::Small)
                .fixed_rect(r_35)
                .show(ui);
            ui.put(r_42, MacInputCell::new("hex_42", &mut u_hex_42).theme(&theme).control_size(MacControlSize::Small));
            MacInputCell::new("dim_100", &mut dim_100).theme(&theme).fixed_rect(r_32_a).show(ui);
            MacInputCell::new("dim_680", &mut dim_680).theme(&theme).fixed_rect(r_32_b).show(ui);
            ui.put(r_32_c, MacInputCell::new("dim_92", &mut dim_92).theme(&theme));
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(r_child_36x18));
            MacInputCell::with_id(egui::Id::unique("patrix_glyph_unicode_36x18"), &mut u_hex_child_36x18)
                .theme(&theme)
                .width(36.0)
                .height(18.0)
                .show(&mut child_ui);
        });

        let mut texts = Vec::new();
        for cs in &shapes {
            common::collect_text_shapes(&cs.shape, &mut texts);
        }
        assert_eq!(texts.len(), 6, "All 6 compact NIB fields (including 36x18 child-Ui) must emit text shapes");

        for (t, field_rect, label) in [
            (texts[0], r_35, "0048 in 35x16"),
            (texts[1], r_42, "0048 in 42x16"),
            (texts[2], r_32_a, "100 in 32x16"),
            (texts[3], r_32_b, "680 in 32x16"),
            (texts[4], r_32_c, "92 in 32x16"),
            (texts[5], r_child_36x18, "0048 in 36x18 child-Ui (.width(36).height(18))"),
        ] {
            let inner_w = field_rect.width() - 4.0; // 2.0pt left + 2.0pt right AppKit NSTextFieldCell inset
            assert_eq!(
                t.galley.rows.len(),
                1,
                "Oracle 2 violation ({label}): single-line NSTextFieldCell wrapped text onto {} lines!",
                t.galley.rows.len()
            );
            assert!(
                t.galley.size().x <= inner_w,
                "Oracle 2 violation ({label}): galley width {:.2}pt exceeds inner_text_rect width {:.2}pt",
                t.galley.size().x,
                inner_w
            );
        }

        // Verify visible foreground glyph pixels in all 4 character quarters of the 4-char "0048" fields (including 36x18 child-Ui!)
        let cell_bg = theme.control_background().to_array();
        for (idx, field_rect, label) in [
            (0usize, r_35, "0048 in 35x16"),
            (1usize, r_42, "0048 in 42x16"),
            (5usize, r_child_36x18, "0048 in 36x18 child-Ui"),
        ] {
            let text_span_w = texts[idx].galley.size().x;
            for (q_idx, ch_name) in [(0usize, '0'), (1, '0'), (2, '4'), (3, '8')] {
                let q0 = q_idx as f32 * 0.25;
                let q1 = (q_idx + 1) as f32 * 0.25;
                let ch_rect = Rect::from_min_max(
                    pos2(field_rect.min.x + 2.0 + text_span_w * q0, field_rect.min.y + 2.0),
                    pos2((field_rect.min.x + 2.0 + text_span_w * q1 + 0.5).min(field_rect.max.x - 2.0), field_rect.max.y - 2.0),
                );
                let fg_pixels = fb.count_differing_pixels(ch_rect, cell_bg, 25);
                assert!(
                    fg_pixels >= 4,
                    "Oracle 2 violation ({label}): character #{q_idx} ('{ch_name}') clipped or missing in {:?} (fg_pixels={fg_pixels})",
                    ch_rect
                );
            }
        }

        // Verify visible foreground pixels for all three 32x16 GSDimensions fields ("100", "680", "92")
        for (field_rect, label) in [(r_32_a, "100"), (r_32_b, "680"), (r_32_c, "92")] {
            let inner = field_rect.shrink2(vec2(2.0, 2.0));
            let fg_pixels = fb.count_differing_pixels(inner, cell_bg, 25);
            assert!(fg_pixels >= 10, "Oracle 2 violation (32x16 '{label}'): expected >= 10 foreground pixels, got {fg_pixels}");
        }
    }
}

/// Oracle 3 (`REQ-013-08.3`): `MacThumbnailCell` `NSView.clipsToBounds = YES` Strict Preview & `16.0pt` Footer Isolation.
#[test]
fn oracle_3_mac_thumbnail_cell_clips_to_bounds_and_footer_isolation_framebuffer() {
    let (ctx, theme) = common::headless_ctx(MacAppearance::DarkAqua);
    let cell_rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(64.0, 64.0));
    let footer_h = 16.0f32;

    // Render baseline cell WITHOUT custom_draw first
    let (fb_base, _) = CpuFramebuffer::render(&ctx, vec2(104.0, 104.0), theme.window_background(), |ui| {
        MacThumbnailCell::new()
            .theme(&theme)
            .size(vec2(64.0, 64.0))
            .footer_height(footer_h)
            .paint_in_rect(ui.painter(), cell_rect, false);
    });

    // Now render cell WITH an intentionally oversized custom_draw shape that tries to bleed into the 16pt footer and outside the 64x64 cell!
    let (fb_drawn, _) = CpuFramebuffer::render(&ctx, vec2(104.0, 104.0), theme.window_background(), |ui| {
        MacThumbnailCell::new()
            .theme(&theme)
            .size(vec2(64.0, 64.0))
            .footer_height(footer_h)
            .custom_draw(|painter, _inner_rect| {
                painter.rect_filled(Rect::from_min_size(pos2(0.0, 0.0), vec2(104.0, 104.0)), 0, theme.system_red());
            })
            .paint_in_rect(ui.painter(), cell_rect, false);
    });

    let preview_inner = Rect::from_min_max(pos2(21.0, 21.0), pos2(83.0, 67.0));
    let mut leaked_outside_preview = 0usize;
    let mut painted_inside_preview = 0usize;
    for y in 0..104 {
        for x in 0..104 {
            let p_base = fb_base.pixels[y * 104 + x];
            let p_draw = fb_drawn.pixels[y * 104 + x];
            let changed = p_base != p_draw;
            let pt = pos2(x as f32 + 0.5, y as f32 + 0.5);
            if preview_inner.contains(pt) {
                if changed {
                    painted_inside_preview += 1;
                }
            } else if changed {
                leaked_outside_preview += 1;
            }
        }
    }
    assert!(painted_inside_preview > 500, "Preview interior must be painted");
    assert_eq!(
        leaked_outside_preview, 0,
        "Oracle 3 violation: MacThumbnailCell leaked {leaked_outside_preview} pixels into the 16pt footer bar, 1pt border, or neighbor cells!"
    );

    if let Ok(png_path) = std::env::var("MACVIEW_EXPORT_ORACLES_PNG") {
        let mut store = ShapeBoundsStore {
            vals: BTreeMap::from([
                (BindingId(1), "120".into()),
                (BindingId(2), "-40".into()),
                (BindingId(3), "680".into()),
                (BindingId(4), "700".into()),
            ]),
        };
        let scene = build_inspector_view_gs_shape_bounds_scene();
        let mut u_hex_35 = String::from("0048");
        let mut u_hex_42 = String::from("0048");
        let mut dim_100 = String::from("100");
        let mut dim_680 = String::from("680");
        let mut dim_92 = String::from("92");

        let (fb_sheet, _) = CpuFramebuffer::render(&ctx, vec2(440.0, 150.0), theme.window_background(), |ui| {
            let mut n_ui = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos2(12.0, 12.0), vec2(192.0, 44.0))));
            scene.show(&mut n_ui, &theme, &mut store, LayoutPolicy::ExactAppleNib, u64::MAX, "");
            MacInputCell::new("s_35", &mut u_hex_35)
                .theme(&theme)
                .control_size(MacControlSize::Small)
                .fixed_rect(Rect::from_min_size(pos2(12.0, 82.0), vec2(35.0, 16.0)))
                .show(ui);
            ui.put(
                Rect::from_min_size(pos2(55.0, 82.0), vec2(42.0, 16.0)),
                MacInputCell::new("s_42", &mut u_hex_42).theme(&theme).control_size(MacControlSize::Small),
            );
            MacInputCell::new("s_100", &mut dim_100)
                .theme(&theme)
                .fixed_rect(Rect::from_min_size(pos2(105.0, 82.0), vec2(32.0, 16.0)))
                .show(ui);
            MacInputCell::new("s_680", &mut dim_680)
                .theme(&theme)
                .fixed_rect(Rect::from_min_size(pos2(145.0, 82.0), vec2(32.0, 16.0)))
                .show(ui);
            ui.put(
                Rect::from_min_size(pos2(185.0, 82.0), vec2(32.0, 16.0)),
                MacInputCell::new("s_92", &mut dim_92).theme(&theme),
            );
            let mut c36_ui = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos2(12.0, 108.0), vec2(36.0, 18.0))));
            MacInputCell::with_id(egui::Id::unique("s_child_36x18"), &mut u_hex_35).theme(&theme).width(36.0).height(18.0).show(&mut c36_ui);
            let cell_r = Rect::from_min_size(pos2(260.0, 16.0), vec2(72.0, 72.0));
            MacThumbnailCell::new()
                .theme(&theme)
                .size(vec2(72.0, 72.0))
                .footer_height(16.0)
                .custom_draw(|p, _inner| {
                    p.circle_filled(pos2(296.0, 48.0), 46.0, theme.control_accent());
                })
                .paint_in_rect(ui.painter(), cell_r, false);
        });
        let flat: Vec<u8> = fb_sheet.pixels.iter().flatten().copied().collect();
        if let Some(img) = image::RgbaImage::from_raw(fb_sheet.width as u32, fb_sheet.height as u32, flat) {
            let _ = img.save(png_path);
        }
    }
}
