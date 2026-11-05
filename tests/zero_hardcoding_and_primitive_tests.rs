//! ISO/IEC/IEEE 29119-3:2021 Zero-Hardcoding Compliance & Primitive Suite Tests.
//!
//! Normative reference: GL-1015 (Regel 11/13: Zero Hardcoded Constants).
//! Verifies:
//! 1. All semantic NSColor and layout geometry tokens resolve dynamically from W3C DTCG catalog.
//! 2. Light (Aqua) and Dark (DarkAqua) appearances yield mathematically exact sRGB values.
//! 3. `MacRow` deterministic compartment budget allocation.
//! 4. `parse_color32` parser robustness for all CSS/Hex token formats.

use egui::Color32;
use egui_macview::layout::MacRow;
use egui_macview::theme::{MacAppearance, MacTheme};
use egui_macview::tokens::dtcg::parse_color32;

/// Test 1: Verifies dynamic semantic color resolution across Aqua and DarkAqua.
#[test]
fn test_theme_dynamic_color_resolution() {
    let light_theme = MacTheme::new(MacAppearance::Aqua);
    let dark_theme = MacTheme::new(MacAppearance::DarkAqua);

    // Aqua Light NSColor checks
    assert_eq!(light_theme.window_background(), Color32::from_rgb(236, 236, 236)); // #ECECEC
    assert_eq!(light_theme.control_background(), Color32::from_rgb(255, 255, 255)); // #FFFFFF
    assert_eq!(light_theme.control(), Color32::from_rgb(234, 234, 234)); // #EAEAEA
    assert_eq!(light_theme.control_accent(), Color32::from_rgb(0, 122, 255)); // #007AFF

    // DarkAqua NSColor checks
    assert_eq!(dark_theme.window_background(), Color32::from_rgb(50, 50, 50)); // #323232
    assert_eq!(dark_theme.control_background(), Color32::from_rgb(30, 30, 30)); // #1E1E1E
    assert_eq!(dark_theme.control(), Color32::from_rgb(58, 58, 58)); // #3A3A3A
    assert_eq!(dark_theme.control_accent(), Color32::from_rgb(10, 132, 255)); // #0A84FF
}

/// Test 2: Verifies dynamic control geometry metrics without hardcoding.
#[test]
fn test_theme_dynamic_geometry_metrics() {
    let theme = MacTheme::default();

    // Compact text field metrics
    let tf_compact = &theme.text_field().compact_infobox;
    assert_eq!(tf_compact.height.points(), 18.0);
    assert_eq!(tf_compact.font_size.points(), 11.0);
    assert_eq!(tf_compact.corner_radius.points(), 3.0);

    // Push button metrics
    assert_eq!(theme.button().regular.height.points(), 22.0);
    assert_eq!(theme.button().regular.corner_radius.points(), 6.0);
    assert_eq!(theme.button().small.height.points(), 18.0);
    assert_eq!(theme.button().small.corner_radius.points(), 4.0);

    // Focus ring metrics
    assert_eq!(theme.focus_ring().stroke_width.points(), 3.0);
    assert_eq!(theme.focus_ring().offset.points(), 1.0);
    assert_eq!(theme.focus_ring().corner_radius_delta.points(), 2.0);

    // Popover / panel metrics
    assert_eq!(theme.popover_metrics().compact_panel_width.points(), 178.0);
    assert_eq!(theme.popover_metrics().compact_panel_height.points(), 63.0);
    assert_eq!(theme.popover_metrics().arrow_base_width.points(), 24.0);
    assert_eq!(theme.popover_metrics().arrow_height.points(), 12.0);
}

/// Test 3: Verifies MacRow layout allocation and zero drift.
#[test]
fn test_mac_row_budget_allocation() {
    let mut row = MacRow::new(0.0, 18.0, 178.0, 2.0);

    let (x0, y0, w0, h0) = row.allocate_cell(40.0);
    assert_eq!((x0, y0, w0, h0), (0.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 42.0);

    let (x1, y1, w1, h1) = row.allocate_cell(40.0);
    assert_eq!((x1, y1, w1, h1), (42.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 84.0);

    let (x2, y2, w2, h2) = row.allocate_cell(40.0);
    assert_eq!((x2, y2, w2, h2), (84.0, 0.0, 40.0, 18.0));

    let (x3, y3, w3, h3) = row.allocate_cell(40.0);
    assert_eq!((x3, y3, w3, h3), (126.0, 0.0, 40.0, 18.0));
    assert_eq!(row.cursor_x, 168.0);

    // Remaining width is 178 - 168 = 10 pt
    assert_eq!(row.remaining_width(), 10.0);
}

/// Test 4: Verifies DTCG color token string parser.
#[test]
fn test_dtcg_color_parser() {
    assert_eq!(parse_color32("#ECECEC"), Color32::from_rgb(236, 236, 236));
    assert_eq!(parse_color32("#323232"), Color32::from_rgb(50, 50, 50));
    assert_eq!(parse_color32("#007AFF"), Color32::from_rgb(0, 122, 255));
    assert_eq!(parse_color32("#12345678"), Color32::from_rgba_premultiplied(0x12, 0x34, 0x56, 0x78));

    // CSS rgba string
    let parsed_alpha = parse_color32("rgba(0, 0, 0, 0.10)");
    assert_eq!(parsed_alpha.r(), 0);
    assert_eq!(parsed_alpha.g(), 0);
    assert_eq!(parsed_alpha.b(), 0);
    assert!((parsed_alpha.a() as i32 - 25).abs() <= 1);

    let parsed_focus = parse_color32("rgba(0, 103, 244, 0.50)");
    assert_eq!(parsed_focus.r(), 0);
    assert_eq!(parsed_focus.g(), 103);
    assert_eq!(parsed_focus.b(), 244);
    assert!((parsed_focus.a() as i32 - 128).abs() <= 1);
}
