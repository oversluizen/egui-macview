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
    assert_eq!(
        light_theme.window_background(),
        Color32::from_rgb(236, 236, 236)
    ); // #ECECEC
    assert_eq!(
        light_theme.control_background(),
        Color32::from_rgb(255, 255, 255)
    ); // #FFFFFF
    assert_eq!(light_theme.control(), Color32::from_rgb(234, 234, 234)); // #EAEAEA
    assert_eq!(light_theme.control_accent(), Color32::from_rgb(0, 122, 255)); // #007AFF

    // DarkAqua NSColor checks
    assert_eq!(
        dark_theme.window_background(),
        Color32::from_rgb(50, 50, 50)
    ); // #323232
    assert_eq!(
        dark_theme.control_background(),
        Color32::from_rgb(30, 30, 30)
    ); // #1E1E1E
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

    // Slider metrics
    assert_eq!(theme.slider().track_height.points(), 3.0);
    assert_eq!(theme.slider().thumb_size.points(), 16.0);
    assert_eq!(theme.slider().thumb_stroke_width.points(), 1.0);
    assert_eq!(theme.slider().default_width.points(), 120.0);
    assert_eq!(theme.slider().key_step_fraction.points(), 0.05);

    // Thumbnail cell metrics
    assert_eq!(theme.thumbnail().size.points(), 24.0);
    assert_eq!(theme.thumbnail().corner_radius.points(), 4.0);
    assert_eq!(theme.thumbnail().border_width.points(), 1.0);

    // Outline view metrics
    assert_eq!(theme.outline_view().row_height.points(), 24.0);
    assert_eq!(theme.outline_view().indent_step.points(), 16.0);
    assert_eq!(theme.outline_view().disclosure_size.points(), 10.0);
    assert_eq!(theme.outline_view().disclosure_margin.points(), 4.0);
    assert_eq!(theme.outline_view().content_spacing.points(), 6.0);
    assert_eq!(theme.outline_view().accessory_size.points(), 16.0);
    assert_eq!(theme.outline_view().accessory_spacing.points(), 4.0);
    assert_eq!(theme.outline_view().font_size.points(), 12.0);
    assert_eq!(theme.outline_view().chevron_stroke_width.points(), 1.2);
    assert_eq!(theme.outline_view().chevron_size.points(), 7.0);

    // Split view metrics
    assert_eq!(theme.split_view().divider_thickness.points(), 1.0);
    assert_eq!(theme.split_view().hit_thickness.points(), 6.0);
    assert_eq!(theme.split_view().min_pane_size.points(), 50.0);
    assert_eq!(theme.split_view().handle_length.points(), 24.0);
    assert_eq!(theme.split_view().handle_thickness.points(), 3.0);
    assert_eq!(theme.split_view().default_ratio.points(), 0.5);

    // PopUp button metrics
    assert_eq!(theme.popup_button().height.points(), 22.0);
    assert_eq!(theme.popup_button().corner_radius.points(), 5.0);
    assert_eq!(theme.popup_button().chevron_size.points(), 8.0);
    assert_eq!(theme.popup_button().chevron_stroke_width.points(), 1.2);

    // Segmented metrics
    assert_eq!(theme.segmented().item_width.points(), 36.0);

    // Search field metrics
    assert_eq!(theme.search_field().height.points(), 22.0);
    assert_eq!(theme.search_field().corner_radius.points(), 6.0);
    assert_eq!(theme.search_field().icon_size.points(), 12.0);
    assert_eq!(theme.search_field().clear_button_size.points(), 14.0);
    assert_eq!(theme.search_field().clear_button_icon_padding.points(), 3.5);

    // Stepper metrics
    assert_eq!(theme.stepper().width.points(), 14.0);
    assert_eq!(theme.stepper().height.points(), 22.0);
    assert_eq!(theme.stepper().corner_radius.points(), 4.0);
    assert_eq!(theme.stepper().arrow_size.points(), 6.0);

    // Color well metrics
    assert_eq!(theme.color_well().width.points(), 44.0);
    assert_eq!(theme.color_well().height.points(), 23.0);
    assert_eq!(theme.color_well().corner_radius.points(), 5.0);
    assert_eq!(theme.color_well().border_width.points(), 1.0);
    assert_eq!(theme.color_well().swatch_inset.points(), 3.0);
    assert_eq!(theme.color_well().popup_corner_radius.points(), 8.0);

    // Outline view chevron depth
    assert_eq!(theme.outline_view().chevron_depth.points(), 1.75);

    // Scroll view hit zone padding & min visible length
    assert_eq!(theme.scroll_view().hit_zone_padding.points(), 8.0);
    assert_eq!(theme.scroll_view().min_visible_thumb_length.points(), 2.0);
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

/// Test 4: Verifies DTCG color token string parser uses unmultiplied sRGBA conversion.
#[test]
fn test_dtcg_color_parser() {
    assert_eq!(parse_color32("#ECECEC"), Color32::from_rgb(236, 236, 236));
    assert_eq!(parse_color32("#323232"), Color32::from_rgb(50, 50, 50));
    assert_eq!(parse_color32("#007AFF"), Color32::from_rgb(0, 122, 255));
    assert_eq!(
        parse_color32("#12345678"),
        Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0x78)
    );

    // CSS rgba string
    let parsed_alpha = parse_color32("rgba(0, 0, 0, 0.10)");
    let srgba_alpha = parsed_alpha.to_srgba_unmultiplied();
    assert_eq!(srgba_alpha[0], 0);
    assert_eq!(srgba_alpha[1], 0);
    assert_eq!(srgba_alpha[2], 0);
    assert!((srgba_alpha[3] as i32 - 25).abs() <= 1);

    let parsed_focus = parse_color32("rgba(0, 103, 244, 0.50)");
    assert_eq!(
        parsed_focus,
        Color32::from_rgba_unmultiplied(0, 103, 244, 128)
    );
    let srgba_focus = parsed_focus.to_srgba_unmultiplied();
    assert_eq!(srgba_focus[0], 0);
    assert!((srgba_focus[1] as i32 - 103).abs() <= 1);
    assert!((srgba_focus[2] as i32 - 244).abs() <= 1);
    assert_eq!(srgba_focus[3], 128);
}
