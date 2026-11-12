//! ISO/IEC/IEEE 29119-3:2021 Zero-Hardcoding Compliance & Primitive Suite Tests.
//!
//! Normative reference: Design Token Driven Architecture (Zero Magic Numbers & Zero Hardcoded Constants).
//! Verifies:
//! 1. All semantic NSColor and layout geometry tokens resolve dynamically from W3C DTCG catalog.
//! 2. Light (Aqua) and Dark (DarkAqua) appearances yield mathematically exact sRGB values.
//! 3. `MacRow` deterministic compartment budget allocation.
//! 4. `parse_color32` parser robustness for all CSS/Hex token formats.
//! 5. Zero `MacNibPrimitives`, raw `[u8; 3]` RGB arrays, or fake rectangle-painting helpers across all of `src/`.

mod guard_helpers;

use egui::Color32;
use egui_macview::layout::MacRow;
use egui_macview::theme::{MacAppearance, MacTheme};
use egui_macview::tokens::dtcg::parse_color32;
use guard_helpers::{
    check_hardcoded_colors_in_source, check_magic_numbers_in_source, collect_rs_files,
    strip_comments_and_strings,
};
use std::fs;
use std::path::Path;

/// Test 1: Verifies dynamic semantic color resolution across Aqua and DarkAqua.
#[test]
fn test_theme_dynamic_color_resolution() {
    let light_theme = MacTheme::new(MacAppearance::Aqua);
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);

    assert_eq!(light_theme.window_background(), Color32::from_rgb(236, 236, 236));
    assert_eq!(light_theme.control_background(), Color32::from_rgb(255, 255, 255));
    assert_eq!(light_theme.control(), Color32::from_rgb(234, 234, 234));
    assert_eq!(light_theme.control_accent(), Color32::from_rgb(0, 122, 255));

    assert_eq!(dark_theme.window_background(), Color32::from_rgb(50, 50, 50));
    assert_eq!(dark_theme.control_background(), Color32::from_rgb(30, 30, 30));
    assert_eq!(dark_theme.control(), Color32::from_rgb(58, 58, 58));
    assert_eq!(dark_theme.control_accent(), Color32::from_rgb(10, 132, 255));
}

/// Test 2: Verifies dynamic control geometry metrics without hardcoding.
#[test]
fn test_theme_dynamic_geometry_metrics() {
    let theme = MacTheme::default();
    let tf_compact = &theme.text_field().compact_infobox;
    assert_eq!((tf_compact.height.points(), tf_compact.font_size.points(), tf_compact.corner_radius.points()), (18.0, 11.0, 2.0));
    assert_eq!((theme.button().regular.height.points(), theme.button().regular.corner_radius.points()), (22.0, 6.0));
    assert_eq!((theme.button().small.height.points(), theme.button().small.corner_radius.points()), (18.0, 4.0));
    assert_eq!((theme.focus_ring().stroke_width.points(), theme.focus_ring().offset.points(), theme.focus_ring().corner_radius_delta.points()), (3.0, 1.0, 2.0));
    assert_eq!((theme.popover_metrics().compact_panel_width.points(), theme.popover_metrics().compact_panel_height.points(), theme.popover_metrics().arrow_base_width.points(), theme.popover_metrics().arrow_height.points()), (178.0, 63.0, 24.0, 12.0));
    assert_eq!((theme.slider().track_height.points(), theme.slider().thumb_size.points(), theme.slider().thumb_stroke_width.points(), theme.slider().default_width.points(), theme.slider().key_step_fraction.points()), (3.0, 16.0, 1.0, 120.0, 0.05));
    assert_eq!((theme.thumbnail().size.points(), theme.thumbnail().corner_radius.points(), theme.thumbnail().border_width.points()), (24.0, 4.0, 1.0));
    assert_eq!((theme.outline_view().row_height.points(), theme.outline_view().indent_step.points(), theme.outline_view().disclosure_size.points(), theme.outline_view().disclosure_margin.points(), theme.outline_view().content_spacing.points(), theme.outline_view().accessory_size.points(), theme.outline_view().accessory_spacing.points(), theme.outline_view().font_size.points(), theme.outline_view().chevron_stroke_width.points(), theme.outline_view().chevron_size.points(), theme.outline_view().chevron_depth.points()), (24.0, 16.0, 10.0, 4.0, 6.0, 16.0, 4.0, 12.0, 1.2, 7.0, 1.75));
    assert_eq!((theme.split_view().divider_thickness.points(), theme.split_view().hit_thickness.points(), theme.split_view().min_pane_size.points(), theme.split_view().handle_length.points(), theme.split_view().handle_thickness.points(), theme.split_view().default_ratio.points()), (1.0, 6.0, 50.0, 24.0, 3.0, 0.5));
    assert_eq!((theme.popup_button().height.points(), theme.popup_button().corner_radius.points(), theme.popup_button().chevron_size.points(), theme.popup_button().chevron_stroke_width.points()), (22.0, 5.0, 8.0, 1.25));
    assert_eq!(theme.segmented().item_width.points(), 36.0);
    assert_eq!((theme.search_field().height.points(), theme.search_field().corner_radius.points(), theme.search_field().icon_size.points(), theme.search_field().clear_button_size.points(), theme.search_field().clear_button_icon_padding.points()), (22.0, 6.0, 12.0, 14.0, 3.5));
    assert_eq!((theme.stepper().width.points(), theme.stepper().height.points(), theme.stepper().corner_radius.points(), theme.stepper().arrow_size.points()), (14.0, 22.0, 4.0, 6.0));
    assert_eq!((theme.color_well().width.points(), theme.color_well().height.points(), theme.color_well().corner_radius.points(), theme.color_well().border_width.points(), theme.color_well().swatch_inset.points(), theme.color_well().popup_corner_radius.points()), (44.0, 23.0, 5.0, 1.0, 3.0, 8.0));
    assert_eq!((theme.scroll_view().hit_zone_padding.points(), theme.scroll_view().min_visible_thumb_length.points()), (8.0, 2.0));
    assert_eq!((theme.progress_indicator().regular.bar_height.points(), theme.progress_indicator().small.bar_height.points(), theme.progress_indicator().mini.bar_height.points(), theme.progress_indicator().spoke_count.points()), (6.0, 4.0, 4.0, 12.0));
    assert_eq!((theme.table_view().header_height.points(), theme.table_view().row_height.points()), (28.0, 24.0));
    assert_eq!((theme.token_field().min_height.points(), theme.token_field().pill_height.points()), (22.0, 16.0));
    assert_eq!((theme.level_indicator().thickness.points(), theme.level_indicator().default_segment_count.points()), (16.0, 12.0));
    assert_eq!((theme.trim_scrubber().height.points(), theme.trim_scrubber().handle_width.points(), theme.path_control().height.points()), (44.0, 12.0, 22.0));
}

/// Test 3: Verifies MacRow layout allocation and zero drift.
#[test]
fn test_mac_row_budget_allocation() {
    let mut row = MacRow::new(0.0, 18.0, 178.0, 2.0);
    assert_eq!(row.allocate_cell(40.0), (0.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 42.0);
    assert_eq!(row.allocate_cell(40.0), (42.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 84.0);
    assert_eq!(row.allocate_cell(40.0), (84.0, 0.0, 40.0, 18.0));
    assert_eq!(row.allocate_cell(40.0), (126.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 168.0);
    assert_eq!(row.remaining_width(), 10.0);
}

/// Test 4: Verifies DTCG color token string parser uses unmultiplied sRGBA conversion.
#[test]
fn test_dtcg_color_parser() {
    assert_eq!(parse_color32("#ECECEC"), Color32::from_rgb(236, 236, 236));
    assert_eq!(parse_color32("#323232"), Color32::from_rgb(50, 50, 50));
    assert_eq!(parse_color32("#007AFF"), Color32::from_rgb(0, 122, 255));
    assert_eq!(parse_color32("#12345678"), Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0x78));

    let srgba_alpha = parse_color32("rgba(0, 0, 0, 0.10)").to_srgba_unmultiplied();
    assert_eq!((srgba_alpha[0], srgba_alpha[1], srgba_alpha[2]), (0, 0, 0));
    assert!((srgba_alpha[3] as i32 - 25).abs() <= 1);

    let parsed_focus = parse_color32("rgba(0, 103, 244, 0.50)");
    assert_eq!(parsed_focus, Color32::from_rgba_unmultiplied(0, 103, 244, 128));
}

/// Test 5: Verifies consolidated single-struct `IdTypeMap` state per widget (`MacScrollView`, `MacInputCell`)
/// and color-independent `Galley` reuse (`Color32::PLACEHOLDER`) in `MacButtonCell` and `MacSegmentedControl`.
#[test]
fn test_consolidated_widget_memory_and_galley_reuse() {
    use egui_macview::{MacButtonCell, MacInputCell, MacScrollView, MacSegmentedControl};

    let ctx = egui::Context::default();
    let theme = MacTheme::default();
    let mut text = String::from("42.0");

    let base_len = ctx.data(|d| d.len());
    let mut out1 = ctx.run_ui(egui::RawInput::default(), |ui| {
        MacScrollView::new("perf_scroll").theme(&theme).max_height(80.0).show(ui, |c_ui| {
            c_ui.allocate_exact_size(egui::vec2(120.0, 240.0), egui::Sense::hover());
        });
    });
    out1.textures_delta.clear();
    assert_eq!(ctx.data(|d| d.len()) - base_len, 2);

    let before_unfocused_ic = ctx.data(|d| d.len());
    let mut out2 = ctx.run_ui(egui::RawInput::default(), |ui| {
        MacInputCell::new("perf_input_idle", &mut text).theme(&theme).show(ui);
    });
    out2.textures_delta.clear();
    assert_eq!(ctx.data(|d| d.len()) - before_unfocused_ic, 1);

    let focused_id = egui::Id::NULL.with("perf_input_focused");
    ctx.memory_mut(|m| m.request_focus(focused_id));
    let before_focused_ic = ctx.data(|d| d.len());
    let mut out3 = ctx.run_ui(egui::RawInput::default(), |ui| {
        MacInputCell::with_id(focused_id, &mut text).theme(&theme).show(ui);
    });
    out3.textures_delta.clear();
    assert_eq!(ctx.data(|d| d.len()) - before_focused_ic, 2);

    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        ui.add(MacButtonCell::new("Action").theme(&theme));
        MacSegmentedControl::new(&["One", "Two"], 0).theme(&theme).show(ui, |_| {});
    });
    out.textures_delta.clear();
    let mut text_shapes = 0usize;
    for clipped in &out.shapes {
        if let egui::epaint::Shape::Text(ts) = &clipped.shape {
            text_shapes += 1;
            for section in &ts.galley.job.sections {
                assert_eq!(section.format.color, Color32::PLACEHOLDER);
            }
        }
    }
    assert_eq!(text_shapes, 3);
}

/// Test 6: Scans all of `src/` (including `src/surfaces/`) to enforce zero `MacNibPrimitives`,
/// zero raw `[u8; 3]` RGB arrays, zero `Color32::from_rgb` outside `dtcg.rs`, zero integer-to-float
/// literal bypass functions (`fn pt(`), and zero fake rectangle-painting helpers (`rect_box`, `sharp_input`).
#[test]
fn test_zero_nib_primitives_and_raw_rgb_across_all_src() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !manifest_dir.join("src/surfaces/nib_primitives.rs").exists(),
        "src/surfaces/nib_primitives.rs (MacNibPrimitives) must not exist"
    );

    let forbidden_anywhere_in_src = [
        "MacNibPrimitives",
        "nib_primitives::MacNibPrimitives",
        "[u8; 3]",
        "rgb_color32",
        "fn rect_box(",
        "fn rect_fill(",
        "fn sharp_input(",
        "fn bezel_edges(",
        "fn no_color_well(",
        "fn table_header(",
        "fn ticked_slider(",
        "fn pt(",
        "fn half_pt(",
        "fn tenth_pt(",
        "let mut checked =",
        "let mut _ =",
    ];

    let mut violations = Vec::new();
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        let content = fs::read_to_string(&file).expect("read src file");
        let stripped = strip_comments_and_strings(&content);
        for (idx, line) in stripped.lines().enumerate() {
            for pat in &forbidden_anywhere_in_src {
                if line.contains(pat) {
                    violations.push(format!("{rel}:{} -> forbidden pattern `{pat}`", idx + 1));
                }
            }
            if rel != "src/tokens/dtcg.rs" && rel != "src/theme/contrast.rs" {
                for bad_color in ["Color32::from_rgb", "Color32::from_rgba", "from_hex"] {
                    if line.contains(bad_color) {
                        violations.push(format!("{rel}:{} -> forbidden `{bad_color}` outside DTCG parser", idx + 1));
                    }
                }
            }
        }
        if rel != "src/tokens/dtcg.rs" && rel != "src/theme/contrast.rs" {
            violations.extend(check_hardcoded_colors_in_source(&rel, &content));
        }
        if ["src/controls/", "src/surfaces/", "src/layout/", "src/navigation/", "src/feedback/"]
            .iter()
            .any(|prefix| rel.starts_with(prefix))
        {
            violations.extend(check_magic_numbers_in_source(&rel, &content));
        }
    }
    assert!(violations.is_empty(), "Zero Hardcoding & NibPrimitives Guard:\n{violations:#?}");
    assert!(!check_hardcoded_colors_in_source("bad.rs", "pub fn fill(rgb: [u8; 3]) {}").is_empty());
    assert!(!check_magic_numbers_in_source("bad.rs", "const fn pt(n: i32) -> f32 { n as f32 }").is_empty());
}
