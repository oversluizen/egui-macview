//! COTS Verification Suite for v0.6.10 AppKit NIB Controls & Surface Primitives.
//!
//! Covers:
//! 1. Removal of `MacNibPrimitives` and full-`src/` hardcoding guard coverage.
//! 2. `MacSlider::tick_marks(usize)` (`NSSlider.numberOfTickMarks`): `4.0pt` neutral track without
//!    accent fill, `1.0x4.0pt` tick marks at `3.0pt` y-offset, and `7.0x15.0pt` (`r=3.5pt`) pill thumb.
//! 3. `MacControlSize` (`Regular`, `Small`, `Mini`) across `MacInputCell`, `MacCheckbox`,
//!    `MacRadioButton`, `MacPopUpButton`, and `MacButtonCell`.
//! 4. `MacSegmentedControl::small_square()` & `.show_multi(...)` (`NSSegmentedSwitchTrackingSelectAny`).
//! 5. `MacTagColorPicker::rows(usize)`, `.dot_size(f32)`, and `.spacing(f32, f32)` (2x6 sidebar layout).
//! 6. `MacPopoverSurface` (`NSPopover` container surface with `8.0pt` corner radius & `1.0pt` border).

mod guard_helpers;

use egui::{pos2, vec2, Color32, Context, Event, PointerButton, Pos2, RawInput, Rect, Shape};
use egui_macview::theme::{MacAppearance, MacTheme};
use egui_macview::{
    MacButtonCell, MacCheckbox, MacControlSize, MacInputCell, MacPopUpButton, MacPopoverSurface,
    MacRadioButton, MacSegmentedControl, MacSlider, MacTagColorPicker,
};
use guard_helpers::{check_hardcoded_colors_in_source, collect_rs_files};
use std::fs;
use std::path::Path;

fn pointer_click_input(pos: Pos2) -> RawInput {
    RawInput {
        events: vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    }
}

#[test]
fn test_v0610_zero_nib_primitives_and_full_src_hardcoding_guard() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!manifest_dir.join("src/surfaces/nib_primitives.rs").exists());
    let surfaces_mod = fs::read_to_string(manifest_dir.join("src/surfaces/mod.rs")).unwrap();
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();
    assert!(!surfaces_mod.contains("MacNibPrimitives") && !surfaces_mod.contains("nib_primitives"));
    assert!(!lib_rs.contains("MacNibPrimitives") && !lib_rs.contains("nib_primitives"));

    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = Color32::from_rgb(1, 2, 3);").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = Color32::from_rgba_unmultiplied(1, 2, 3, 4);").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = from_hex(\"#112233\");").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let raw: [u8; 3] = [0, 0, 0];").is_empty());

    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        if rel != "src/tokens/dtcg.rs" && rel != "src/theme/contrast.rs" {
            let content = fs::read_to_string(&file).unwrap();
            let violations = check_hardcoded_colors_in_source(&rel, &content);
            assert!(violations.is_empty(), "Hardcoding in {rel}: {violations:#?}");
        }
    }
}

#[test]
fn test_v0610_slider_tick_marks_geometry_and_tokens() {
    let light = MacTheme::new(MacAppearance::Aqua);
    let dark = MacTheme::new(MacAppearance::DarkAqua);
    let sm = light.slider();

    assert_eq!(sm.ticked_track_height.points(), 4.0);
    assert_eq!(sm.tick_width.points(), 1.0);
    assert_eq!(sm.tick_height.points(), 4.0);
    assert_eq!(sm.tick_offset.points(), 3.0);
    assert_eq!(sm.ticked_thumb_width.points(), 7.0);
    assert_eq!(sm.ticked_thumb_height.points(), 15.0);
    assert_eq!(sm.ticked_thumb_corner_radius.points(), 3.5);
    assert_eq!(light.slider_tick_mark_color(), Color32::from_rgb(0xB8, 0xB8, 0xB8));
    assert_eq!(dark.slider_tick_mark_color(), Color32::from_rgb(0x66, 0x66, 0x66));

    let ctx = Context::default();
    let mut val = 50.0_f32;
    let mut out_unticked = ctx.run_ui(RawInput::default(), |ui| {
        MacSlider::new(&mut val, 0.0..=100.0).theme(&light).show(ui);
    });
    out_unticked.textures_delta.clear();
    let has_circle_unticked = out_unticked.shapes.iter().any(|s| matches!(s.shape, Shape::Circle(_)));
    let has_accent_unticked = out_unticked.shapes.iter().any(|s| match &s.shape {
        Shape::Rect(r) => r.fill == light.control_accent(),
        _ => false,
    });
    assert!(has_circle_unticked && has_accent_unticked);

    let rect = Rect::from_min_size(pos2(20.0, 20.0), vec2(120.0, 22.0));
    let mut out_ticked = ctx.run_ui(RawInput::default(), |ui| {
        MacSlider::new(&mut val, 0.0..=100.0)
            .theme(&light)
            .tick_marks(11)
            .fixed_rect(rect)
            .show(ui);
    });
    out_ticked.textures_delta.clear();
    let has_circle_ticked = out_ticked.shapes.iter().any(|s| matches!(s.shape, Shape::Circle(_)));
    let has_accent_ticked = out_ticked.shapes.iter().any(|s| match &s.shape {
        Shape::Rect(r) => r.fill == light.control_accent(),
        _ => false,
    });
    assert!(!has_circle_ticked, "Ticked slider must use vertical pill thumb instead of circle");
    assert!(!has_accent_ticked, "Ticked slider must not paint blue accent fill on track");

    let tick_color = light.slider_tick_mark_color();
    let tick_lines: Vec<_> = out_ticked
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::LineSegment { points, stroke } if stroke.color == tick_color => Some((*points, *stroke)),
            _ => None,
        })
        .collect();
    assert_eq!(tick_lines.len(), 11, "Expected 11 vertical tick marks");
    for (pts, stroke) in &tick_lines {
        assert!((stroke.width - 1.0).abs() < 1e-3);
        assert!(((pts[1].y - pts[0].y) - 4.0).abs() < 1e-3);
    }

    let pill_thumbs: Vec<_> = out_ticked
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            Shape::Rect(r)
                if r.fill == light.slider_thumb_bg()
                    && (r.rect.width() - 7.0).abs() < 1e-3
                    && (r.rect.height() - 15.0).abs() < 1e-3 =>
            {
                Some(r.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(pill_thumbs.len(), 1, "Expected 7x15pt pill thumb");
}

#[test]
fn test_v0610_control_size_regular_small_mini_across_five_controls() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    let ctx = Context::default();

    let mut text = String::from("Sample");
    let mut checked = true;
    let mut radio = true;
    let mut popup_idx = 0usize;
    let items = ["One", "Two"];

    for (idx, (size, exp_btn_h, exp_box_sz, exp_font)) in [
        (MacControlSize::Regular, 22.0_f32, 14.0_f32, 13.0_f32),
        (MacControlSize::Small, 18.0_f32, 12.0_f32, 11.0_f32),
        (MacControlSize::Mini, 15.0_f32, 10.0_f32, 9.0_f32),
    ]
    .into_iter()
    .enumerate()
    {
        let mut btn_h = 0.0_f32;
        let mut pop_h = 0.0_f32;
        let mut inp_h = 0.0_f32;
        let mut chk_h = 0.0_f32;
        let mut rad_h = 0.0_f32;
        let mut out = ctx.run_ui(RawInput::default(), |ui| {
            inp_h = MacInputCell::new(("inp", idx), &mut text).theme(&theme).control_size(size).show(ui).rect.height();
            btn_h = ui.add(MacButtonCell::new("Button").theme(&theme).control_size(size)).rect.height();
            pop_h = MacPopUpButton::new(&mut popup_idx, &items).theme(&theme).control_size(size).show(ui).rect.height();
            chk_h = ui.add(MacCheckbox::new(&mut checked, "Check").theme(&theme).control_size(size)).rect.height();
            rad_h = ui.add(MacRadioButton::new(&mut radio, "Radio").theme(&theme).control_size(size)).rect.height();
        });
        out.textures_delta.clear();
        assert!((btn_h - exp_btn_h).abs() < 1e-3, "MacButtonCell height for {size:?}");
        assert!((pop_h - exp_btn_h).abs() < 1e-3, "MacPopUpButton height for {size:?}");
        let expected_inp_h = if size == MacControlSize::Small { 19.0 } else { exp_btn_h };
        assert!((inp_h - expected_inp_h).abs() < 1e-3, "MacInputCell height for {size:?}");
        assert!(chk_h >= exp_box_sz && rad_h >= exp_box_sz && exp_font > 0.0);
        let has_radio_circle = out.shapes.iter().any(|s| matches!(s.shape, Shape::Circle(_)));
        assert!(has_radio_circle, "MacRadioButton must render circular radio button");
    }
}

#[test]
fn test_v0610_segmented_control_small_square_and_show_multi() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    let ctx = Context::default();
    let labels = ["Outline", "Nodes", "Handles", "Metric", "Hints", "Guides"];
    let mut active = [true, false, false, true, false, false];
    let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(180.0, 22.0));

    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        let clicked = MacSegmentedControl::new(&labels, 0)
            .theme(&theme)
            .small_square()
            .fixed_rect(rect)
            .show_multi(ui, &mut active);
        assert_eq!(clicked, None);
    });
    out0.textures_delta.clear();
    let divider_count = out0
        .shapes
        .iter()
        .filter(|s| matches!(s.shape, Shape::LineSegment { .. }))
        .count();
    assert_eq!(divider_count, 5, "6 segments in small_square strip must have 5 internal dividers");

    let seg1_center = pos2(10.0 + 45.0, 21.0);
    let mut clicked_idx = None;
    let mut out1 = ctx.run_ui(pointer_click_input(seg1_center), |ui| {
        clicked_idx = MacSegmentedControl::new(&labels, 0)
            .theme(&theme)
            .small_square()
            .fixed_rect(rect)
            .show_multi(ui, &mut active);
    });
    out1.textures_delta.clear();
    assert_eq!(clicked_idx, Some(1));
    assert_eq!(active, [true, true, false, true, false, false]);

    // Edge case: short `active` slice and empty `options` slice in `show_multi`, plus `control_size` & `fixed_rect` in `show`
    let mut short_active = [false, false];
    let seg5_center = pos2(10.0 + 165.0, 21.0);
    let mut short_clicked = None;
    let mut out2 = ctx.run_ui(pointer_click_input(seg5_center), |ui| {
        short_clicked = MacSegmentedControl::new(&labels, 0)
            .theme(&theme)
            .small_square()
            .fixed_rect(rect)
            .show_multi(ui, &mut short_active);
        let empty_clicked = MacSegmentedControl::new(&[], 0)
            .theme(&theme)
            .show_multi(ui, &mut []);
        assert_eq!(empty_clicked, None);
        let small_resp = MacSegmentedControl::new(&labels[..2], 0)
            .theme(&theme)
            .control_size(MacControlSize::Small)
            .small_square()
            .show(ui, |_| {});
        assert!((small_resp.rect.height() - 18.0).abs() < 1e-3);
    });
    out2.textures_delta.clear();
    assert_eq!(short_clicked, None, "Out-of-bounds segment click must not report toggle");
    assert_eq!(short_active, [false, false]);
}

#[test]
fn test_v0610_tag_color_picker_multi_row_layout() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    let ctx = Context::default();
    let mut selected = Some(0usize);
    let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(186.0, 42.0));

    let mut out0 = ctx.run_ui(RawInput::default(), |ui| {
        MacTagColorPicker::new(&mut selected)
            .theme(&theme)
            .rows(2)
            .dot_size(12.0)
            .spacing(6.0, 4.0)
            .fixed_rect(rect)
            .show(ui);
    });
    out0.textures_delta.clear();

    let c0 = MacTagColorPicker::swatch_center_grid(rect, 0, 12, 6, 2, 12.0, 6.0, 4.0);
    let c6 = MacTagColorPicker::swatch_center_grid(rect, 6, 12, 6, 2, 12.0, 6.0, 4.0);
    let c7 = MacTagColorPicker::swatch_center_grid(rect, 7, 12, 6, 2, 12.0, 6.0, 4.0);
    assert!((c0.x - c6.x).abs() < 1e-3, "Column 0 aligned across rows 0 and 1");
    assert!(((c7.x - c6.x) - 18.0).abs() < 1e-3, "Column step = 12pt + 6pt = 18pt");
    assert!(((c6.y - c0.y) - 16.0).abs() < 1e-3, "Row step = 12pt + 4pt = 16pt");

    let mut out1 = ctx.run_ui(pointer_click_input(c7), |ui| {
        let resp = MacTagColorPicker::new(&mut selected)
            .theme(&theme)
            .rows(2)
            .dot_size(12.0)
            .spacing(6.0, 4.0)
            .fixed_rect(rect)
            .show(ui);
        assert!(resp.changed());
    });
    out1.textures_delta.clear();
    assert_eq!(selected, Some(7));

    let mut default_rows_size = egui::Vec2::ZERO;
    let mut zero_bounds_size = egui::Vec2::ZERO;
    let mut out2 = ctx.run_ui(RawInput::default(), |ui| {
        default_rows_size = MacTagColorPicker::new(&mut selected)
            .theme(&theme)
            .rows(2)
            .show(ui)
            .rect
            .size();
        zero_bounds_size = MacTagColorPicker::new(&mut selected)
            .theme(&theme)
            .columns(0)
            .rows(0)
            .show(ui)
            .rect
            .size();
    });
    out2.textures_delta.clear();
    assert!((default_rows_size.x - 6.0 * (12.0 + 6.0)).abs() < 1e-3);
    assert!((default_rows_size.y - 2.0 * (12.0 + 4.0)).abs() < 1e-3);
    assert!(zero_bounds_size.x > 0.0 && zero_bounds_size.y > 0.0);
}

#[test]
fn test_v0610_popover_surface_geometry_tokens_and_vibrancy() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    assert_eq!(theme.popover_metrics().surface_corner_radius.points(), 8.0);
    assert_eq!(theme.popover_metrics().surface_border_width.points(), 1.0);

    let ctx = Context::default();
    let rect = Rect::from_min_size(pos2(16.0, 16.0), vec2(220.0, 96.0));
    let mut inner_surface = Color32::TRANSPARENT;
    let mut out = ctx.run_ui(RawInput::default(), |ui| {
        MacPopoverSurface::new()
            .theme(&theme)
            .fixed_rect(rect)
            .show(ui, |inner_ui| {
                inner_surface = theme.active_surface(inner_ui);
            });
    });
    out.textures_delta.clear();
    assert_eq!(inner_surface, theme.popover_background());
    let has_popover_rect = out.shapes.iter().any(|s| match &s.shape {
        Shape::Rect(r) => r.fill == theme.popover_background() && (r.rect.width() - 220.0).abs() < 1e-3,
        _ => false,
    });
    assert!(has_popover_rect, "MacPopoverSurface must paint popover_background rect");
}
