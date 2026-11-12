//! ISO/IEC/IEEE 29119-3:2021 Red COTS Test Baseline for Sprint 1.
//!
//! Normative reference: SPEC-001 Section 1, 2.1, 4.1 & Architecture Fitness Standards.

use std::fs;
use std::path::Path;

use egui::{Context, RawInput};
use egui_macview::layout::{subpixel_snap, subpixel_snap_pos, subpixel_snap_rect, MacForm};
use egui_macview::motion::CASpring;
use egui_macview::theme::{MacAppearance, MacTheme};
use egui_macview::tokens::{sf_pro_tracking, MacDesignTokens};

/// Test 1: Confirms that `design_tokens.json` parses completely into typed structs without error.
#[test]
fn test_token_catalog_deserialization() {
    let tokens = MacDesignTokens::load_default();

    assert_eq!(tokens.name, "egui-macview Design Tokens");
    assert_eq!(tokens.version, "0.6.11");
    assert_eq!(egui_macview::VERSION, "0.6.11");
    assert!(!tokens.description.is_empty());

    let ctrl = tokens.control_metrics();
    assert_eq!(ctrl.focus_ring.stroke_width.points(), 3.0);
    assert_eq!(ctrl.focus_ring.offset.points(), 1.0);
    assert_eq!(ctrl.focus_ring.corner_radius_delta.points(), 2.0);

    assert_eq!(ctrl.text_field.regular.height.points(), 22.0);
    assert_eq!(ctrl.text_field.regular.font_size.points(), 13.0);
    assert_eq!(ctrl.text_field.regular.corner_radius.points(), 2.0);
    assert_eq!(ctrl.text_field.small.height.points(), 19.0);
    assert_eq!(ctrl.text_field.small.corner_radius.points(), 2.0);
    assert_eq!(ctrl.text_field.compact_infobox.height.points(), 18.0);
    assert_eq!(ctrl.text_field.compact_infobox.corner_radius.points(), 2.0);
    assert_eq!(ctrl.text_field.compact().height.points(), 18.0);
    assert_eq!(ctrl.text_field.mini.height.points(), 15.0);
    assert_eq!(ctrl.text_field.mini.corner_radius.points(), 1.5);
    assert_eq!(ctrl.text_field.large.height.points(), 28.0);

    assert_eq!(ctrl.button.regular.height.points(), 22.0);
    assert_eq!(ctrl.button.small.height.points(), 18.0);
    assert_eq!(ctrl.segmented.regular.height.points(), 22.0);
    assert_eq!(ctrl.segmented.small.height.points(), 18.0);

    let tb = tokens.toolbar_metrics();
    assert_eq!(tb.traffic_lights_inset.points(), 70.0);
    assert_eq!(tb.height_unified.points(), 52.0);
    assert_eq!(tb.height_compact.points(), 44.0);
    assert_eq!(tb.icon_size.points(), 20.0);
    assert_eq!(tb.hit_target_width.points(), 32.0);
    assert_eq!(tb.hit_target_height.points(), 28.0);
    assert_eq!(tb.caption_button_width.points(), 46.0);
    assert_eq!(tb.caption_button_height.points(), 32.0);
    assert_eq!(tb.caption_icon_size.points(), 10.0);
    assert_eq!(tb.traffic_light_radius.points(), 6.0);
    assert_eq!(tb.traffic_light_spacing.points(), 8.0);

    let pop = tokens.popover_metrics();
    assert_eq!(pop.arrow_base_width.points(), 24.0);
    assert_eq!(pop.arrow_height.points(), 12.0);
    assert_eq!(pop.compact_panel_width.points(), 178.0);
    assert_eq!(pop.compact_panel_height.points(), 63.0);
    assert_eq!(pop.wing_width.points(), 84.0);
    assert_eq!(pop.wing_height.points(), 44.0);

    let colors = tokens.colors();
    assert_eq!(colors.aqua.window_background.hex_or_rgba(), "#ECECEC");
    assert_eq!(colors.aqua.control_accent.hex_or_rgba(), "#007AFF");
    assert_eq!(colors.dark_aqua.window_background.hex_or_rgba(), "#323232");
    assert_eq!(colors.dark_aqua.control_accent.hex_or_rgba(), "#0A84FF");

    let typo = tokens.typography();
    assert_eq!(typo.tracking_table.len(), 19);
    assert_eq!(typo.tracking_table.get("pt_6").unwrap().tracking_pt, 0.24);
    assert_eq!(typo.tracking_table.get("pt_12").unwrap().tracking_pt, 0.00);
    assert_eq!(typo.tracking_table.get("pt_20").unwrap().tracking_pt, -0.45);

    let motion = tokens.motion();
    assert_eq!(motion.caspring_default.mass, 1.0);
    assert_eq!(motion.caspring_default.stiffness, 100.0);
    assert_eq!(motion.caspring_default.damping, 10.0);
    assert_eq!(motion.hud_panel_transition.response, 0.35);
    assert_eq!(motion.hud_panel_transition.damping_ratio, 0.85);
}

/// Test 2: Asserts CASpring analytical precision against Apple CoreAnimation parameters.
#[test]
fn test_caspring_analytical_precision() {
    let tolerance = 1e-4;
    let spring_default = CASpring::from_frequency(10.0, 0.5);
    assert!((spring_default.omega_0() - 10.0).abs() < tolerance);
    assert!((spring_default.omega_d() - 8.660254).abs() < tolerance);
    assert_eq!(spring_default.damping_ratio, 0.5);
    assert_eq!(spring_default.evaluate(0.0), 0.0);
    assert_eq!(spring_default.velocity(0.0), 0.0);

    let t_settle_def = spring_default.settling_duration();
    assert!((t_settle_def - 1.381551).abs() < 1e-3);
    assert!((spring_default.evaluate(t_settle_def) - 1.0).abs() <= 0.001);

    let spring_phys = CASpring::from_physical(1.0, 100.0, 10.0);
    assert!((spring_phys.omega_0() - 10.0).abs() < tolerance);
    assert!((spring_phys.omega_d() - 8.660254).abs() < tolerance);
    assert_eq!(spring_phys.damping_ratio, 0.5);

    let hud_spring = CASpring::new(0.35, 0.85);
    assert!((hud_spring.omega_0() - 17.951958).abs() < tolerance);
    assert!((hud_spring.omega_d() - 9.456781).abs() < tolerance);
    assert_eq!(hud_spring.damping_ratio, 0.85);
    assert_eq!(hud_spring.evaluate(0.0), 0.0);
    assert_eq!(hud_spring.velocity(0.0), 0.0);

    let t_settle_hud = hud_spring.settling_duration();
    assert!((t_settle_hud - 0.453).abs() < 0.002);
    assert!((hud_spring.evaluate(t_settle_hud) - 1.0).abs() <= 0.002);
    assert!((hud_spring.evaluate(0.50) - 1.0).abs() <= 0.001);
}

/// Test 3: Asserts compact HUD panel budget and zero-clipping guarantee (SPEC-001 Section 4.1).
#[test]
fn test_compact_hud_panel_budget_zero_clipping() {
    let tokens = MacDesignTokens::load_default();
    let total_panel_height = tokens.popover_metrics().compact_panel_height.points();
    let header_height = tokens.control_metrics().text_field.regular.height.points();
    assert_eq!((total_panel_height, header_height), (63.0, 22.0));

    let available_body_budget = total_panel_height - header_height;
    assert_eq!(available_body_budget, 41.0);

    let cell_height = tokens.control_metrics().text_field.compact_infobox.height.points();
    assert_eq!(cell_height, 18.0);
    let claimed_body_height = cell_height + 1.0 + cell_height;
    assert_eq!(claimed_body_height, 37.0);
    assert!(claimed_body_height <= available_body_budget);
    assert_eq!(available_body_budget - claimed_body_height, 4.0);
}

/// Test 4: Tests subpixel snap criterion on Windows 150% fractional scaling (SPEC-001 Section 2.1 A).
#[test]
fn test_subpixel_snap_windows_150_percent() {
    let scale = 1.5;
    let test_coords = [0.0, 0.25, 0.333, 0.5, 0.667, 1.0, 10.1, 10.4, 25.75, 41.333];
    for &x_logical in &test_coords {
        let x_snapped = subpixel_snap(x_logical, scale);
        let hardware_pixels = x_snapped * scale;
        assert!((hardware_pixels - hardware_pixels.round()).abs() < 1e-4);
    }

    let p = egui::Pos2::new(10.1, 20.4);
    let snapped_p = subpixel_snap_pos(p, scale);
    assert!(((snapped_p.x * scale) - (snapped_p.x * scale).round()).abs() < 1e-4);
    assert!(((snapped_p.y * scale) - (snapped_p.y * scale).round()).abs() < 1e-4);

    let r = egui::Rect::from_min_max(egui::Pos2::new(5.33, 7.15), egui::Pos2::new(55.85, 87.9));
    let snapped_r = subpixel_snap_rect(r, scale);
    assert!(((snapped_r.min.x * scale) - (snapped_r.min.x * scale).round()).abs() < 1e-4);
    assert!(((snapped_r.max.y * scale) - (snapped_r.max.y * scale).round()).abs() < 1e-4);
}

/// Test 5: Validates SF Pro dynamic optical tracking cubic polynomial against Apple HIG table.
#[test]
fn test_sf_pro_dynamic_tracking_optical_bounds() {
    let tokens = MacDesignTokens::load_default();
    let typo = tokens.typography();

    assert_eq!(typo.tracking_table.get("pt_6").unwrap().tracking_pt, 0.24);
    assert_eq!(typo.tracking_table.get("pt_12").unwrap().tracking_pt, 0.00);
    assert_eq!(typo.tracking_table.get("pt_20").unwrap().tracking_pt, -0.45);

    let (track_6, track_12, track_20) = (typo.tracking_for_size(6.0), typo.tracking_for_size(12.0), typo.tracking_for_size(20.0));
    assert!(track_6 > 0.15 && (track_6 - 0.24).abs() < 0.05);
    assert!(track_12.abs() < 0.02);
    assert!(track_20 < -0.35 && (track_20 - (-0.45)).abs() < 0.05);
    assert!(track_6 > track_12 && track_12 > track_20);

    assert!((sf_pro_tracking(6.0) as f64 - track_6).abs() < 1e-4);
    assert!((sf_pro_tracking(12.0) as f64 - track_12).abs() < 1e-4);
    assert!((sf_pro_tracking(20.0) as f64 - track_20).abs() < 1e-4);
}

/// Test 6: Verifies 1-to-1 version synchronization across Cargo.toml, CHANGELOG.md, design_tokens.json, and all specs in specs/.
#[test]
fn test_version_synchronization_invariants() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(cargo_toml.contains("version = \"0.6.11\""), "Cargo.toml must have version = \"0.6.11\"");
    assert_eq!(egui_macview::VERSION, "0.6.11");

    let tokens = MacDesignTokens::load_default();
    assert_eq!(tokens.version, "0.6.11");

    let changelog = fs::read_to_string(manifest_dir.join("CHANGELOG.md")).expect("read CHANGELOG.md");
    for header in [
        "## [0.6.11] - 2026-11-12",
        "## [0.6.10] - 2026-11-12",
        "## [0.6.9] - 2026-11-11",
        "## [0.6.8] - 2026-11-11",
        "## [0.6.7] - 2026-11-11",
        "## [0.6.6] - 2026-11-11",
        "## [0.6.5] - 2026-11-11",
        "## [0.6.4] - 2026-11-10",
        "## [0.6.3] - 2026-11-10",
        "## [0.6.2] - 2026-11-10",
        "## [0.6.1] - 2026-11-10",
        "## [0.6.0] - 2026-11-10",
        "## [0.5.2] - 2026-11-10",
        "## [0.5.1] - 2026-11-08",
        "## [0.5.0] - 2026-11-08",
        "## [0.4.1] - 2026-11-08",
        "## [0.4.0] - 2026-11-07",
        "## [0.3.1] - 2026-11-06",
        "## [0.2.0] - 2026-11-06",
        "## [0.1.0] - 2026-11-05",
    ] {
        assert!(changelog.contains(header), "CHANGELOG.md must contain header '{header}'");
    }

    let specs_dir = manifest_dir.join("specs");
    let entries = fs::read_dir(&specs_dir).expect("read specs dir");
    let mut spec_count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            spec_count += 1;
            let content = fs::read_to_string(&path).expect("read spec file");
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            let (expected_ver, expected_milestone) =
                if file_name.starts_with("SPEC-007") || file_name.starts_with("SPEC-009") {
                    ("0.3.0", Some("Milestone 0.3.0"))
                } else if file_name.starts_with("SPEC-010") {
                    ("0.3.1", Some("Milestone 0.3.1"))
                } else if file_name.starts_with("SPEC-008") {
                    ("0.4.0", Some("Milestone 0.4.0"))
                } else if file_name.starts_with("SPEC-011") {
                    ("0.5.0", Some("Milestone 0.5.0"))
                } else if file_name.starts_with("SPEC-012") || file_name.starts_with("SPEC-INDEX") {
                    ("0.6.11", Some("Milestone 0.6.11"))
                } else {
                    ("0.2.0", None)
                };

            assert!(content.contains(&format!("**Document Version** | `{expected_ver}`")), "{file_name} missing `{expected_ver}`");
            if let Some(ms) = expected_milestone {
                assert!(content.contains(ms), "{file_name} missing `{ms}`");
            }
            assert!(content.contains("`0.1.0`") && content.contains("2026-11-05"));
        }
    }
    assert!(spec_count >= 8);
}

/// Test 7: COTS validation of MacForm layout across multiple DPI zoom levels (100%, 125%, 150%).
#[test]
fn test_mac_form_cots_zoom_alignment_and_no_overlap() {
    let theme = MacTheme::new(MacAppearance::Aqua);
    for &zoom in &[1.0f32, 1.25f32, 1.50f32] {
        let ctx = Context::default();
        ctx.set_zoom_factor(zoom);
        let (mut field1, mut field2, mut field3, mut row_rects) = ("Standard text".to_string(), "42".to_string(), true, Vec::new());

        for frame in 0..2 {
            let mut out = ctx.run_ui(RawInput::default(), |ui| {
                MacForm::new("test_cots_form").theme(&theme).show(ui, |form| {
                    let (l1, c1) = form.row_label("Short:", |ui| ui.text_edit_singleline(&mut field1));
                    let (l2, c2) = form.row_label("A Much Longer Form Label:", |ui| ui.text_edit_singleline(&mut field2));
                    let (l3, c3) = form.row_label("Toggle Setting:", |ui| ui.checkbox(&mut field3, "Enable feature"));
                    if frame == 1 {
                        row_rects = vec![(l1.rect, c1.rect), (l2.rect, c2.rect), (l3.rect, c3.rect)];
                    }
                });
            });
            out.textures_delta.clear();
        }
        assert_eq!(row_rects.len(), 3);
        let (right_edge_0, left_edge_1) = (row_rects[0].0.max.x, row_rects[0].1.min.x);
        for (idx, (label_rect, control_rect)) in row_rects.iter().enumerate() {
            assert!((label_rect.max.x - right_edge_0).abs() <= 1.0 && (control_rect.min.x - left_edge_1).abs() <= 1.0 && label_rect.max.x < control_rect.min.x);
            if idx + 1 < row_rects.len() {
                assert!(label_rect.max.y.max(control_rect.max.y) <= row_rects[idx + 1].0.min.y.min(row_rects[idx + 1].1.min.y) + 0.1);
            }
        }
    }
}
