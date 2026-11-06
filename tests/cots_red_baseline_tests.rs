//! ISO/IEC/IEEE 29119-3:2021 Red COTS Test Baseline for Sprint 1.
//!
//! Normative reference: SPEC-001 Section 1, 2.1, 4.1 & GL-1015.
//! Verifies:
//! 1. W3C DTCG Token Catalog Deserialization into strongly typed models.
//! 2. CASpring CoreAnimation Physics analytical precision and decay bounds.
//! 3. Compact HUD panel budget and zero-clipping guarantee.
//! 4. Subpixel snap criterion under fractional Windows DPI scaling (s = 1.50).
//! 5. SF Pro dynamic typography cubic tracking optical bounds against Apple HIG.

use std::fs;
use std::path::Path;

use egui_macview::layout::{subpixel_snap, subpixel_snap_pos, subpixel_snap_rect};
use egui_macview::motion::CASpring;
use egui_macview::tokens::{sf_pro_tracking, MacDesignTokens};

/// Test 1: Confirms that `design_tokens.json` parses completely into typed structs without error.
#[test]
fn test_token_catalog_deserialization() {
    let tokens = MacDesignTokens::load_default();

    // Verify metadata
    assert_eq!(tokens.name, "egui-macview Design Tokens");
    assert_eq!(tokens.version, "0.3.0");
    assert_eq!(egui_macview::VERSION, "0.3.0");
    assert!(!tokens.description.is_empty());

    // Verify Control Metrics
    let ctrl = tokens.control_metrics();
    assert_eq!(ctrl.focus_ring.stroke_width.points(), 3.0);
    assert_eq!(ctrl.focus_ring.offset.points(), 1.0);
    assert_eq!(ctrl.focus_ring.corner_radius_delta.points(), 2.0);

    assert_eq!(ctrl.text_field.regular.height.points(), 22.0);
    assert_eq!(ctrl.text_field.regular.font_size.points(), 13.0);
    assert_eq!(ctrl.text_field.regular.corner_radius.points(), 6.0);
    assert_eq!(ctrl.text_field.small.height.points(), 19.0);
    assert_eq!(ctrl.text_field.compact_infobox.height.points(), 18.0);
    assert_eq!(ctrl.text_field.compact().height.points(), 18.0);
    assert_eq!(ctrl.text_field.mini.height.points(), 15.0);
    assert_eq!(ctrl.text_field.large.height.points(), 28.0);

    assert_eq!(ctrl.button.regular.height.points(), 22.0);
    assert_eq!(ctrl.button.small.height.points(), 18.0);
    assert_eq!(ctrl.segmented.regular.height.points(), 22.0);
    assert_eq!(ctrl.segmented.small.height.points(), 18.0);

    // Verify Toolbar Metrics
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

    // Verify Popover Metrics
    let pop = tokens.popover_metrics();
    assert_eq!(pop.arrow_base_width.points(), 24.0);
    assert_eq!(pop.arrow_height.points(), 12.0);
    assert_eq!(pop.compact_panel_width.points(), 178.0);
    assert_eq!(pop.compact_panel_height.points(), 63.0);
    assert_eq!(pop.wing_width.points(), 84.0);
    assert_eq!(pop.wing_height.points(), 44.0);

    // Verify Colors
    let colors = tokens.colors();
    assert_eq!(colors.aqua.window_background.hex_or_rgba(), "#ECECEC");
    assert_eq!(colors.aqua.control_accent.hex_or_rgba(), "#007AFF");
    assert_eq!(colors.dark_aqua.window_background.hex_or_rgba(), "#323232");
    assert_eq!(colors.dark_aqua.control_accent.hex_or_rgba(), "#0A84FF");

    // Verify Typography Tracking Table
    let typo = tokens.typography();
    assert_eq!(typo.tracking_table.len(), 19);
    assert_eq!(typo.tracking_table.get("pt_6").unwrap().tracking_pt, 0.24);
    assert_eq!(typo.tracking_table.get("pt_12").unwrap().tracking_pt, 0.00);
    assert_eq!(typo.tracking_table.get("pt_20").unwrap().tracking_pt, -0.45);

    // Verify Motion Presets
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

    // 1. caspring_default: mass=1.0, stiffness=100.0, damping=10.0 -> omega_0=10.0, zeta=0.5, omega_d=8.660254
    let spring_default = CASpring::from_frequency(10.0, 0.5);
    assert!(
        (spring_default.omega_0() - 10.0).abs() < tolerance,
        "omega_0 expected 10.0, got {}",
        spring_default.omega_0()
    );
    assert!(
        (spring_default.omega_d() - 8.660254).abs() < tolerance,
        "omega_d expected 8.660254, got {}",
        spring_default.omega_d()
    );
    assert_eq!(spring_default.damping_ratio, 0.5);

    // Initial boundary conditions
    assert_eq!(spring_default.evaluate(0.0), 0.0);
    assert_eq!(spring_default.velocity(0.0), 0.0);

    // Settling duration: -ln(0.001) / (zeta * omega_0) = 6.907755 / 5.0 = 1.381551s
    let t_settle_def = spring_default.settling_duration();
    assert!(
        (t_settle_def - 1.381551).abs() < 1e-3,
        "t_settle expected ~1.38s, got {}",
        t_settle_def
    );
    assert!(
        (spring_default.evaluate(t_settle_def) - 1.0).abs() <= 0.001,
        "Rest tolerance violation at t_settle: |x(t) - 1.0| = {}",
        (spring_default.evaluate(t_settle_def) - 1.0).abs()
    );

    // Verify physical parameterization equivalence (mass=1.0, stiffness=100.0, damping=10.0)
    let spring_phys = CASpring::from_physical(1.0, 100.0, 10.0);
    assert!((spring_phys.omega_0() - 10.0).abs() < tolerance);
    assert!((spring_phys.omega_d() - 8.660254).abs() < tolerance);
    assert_eq!(spring_phys.damping_ratio, 0.5);

    // 2. hud_panel_transition: response=0.35, damping_ratio=0.85 -> omega_0=17.951958, omega_d=9.456781, t_settle~0.453s
    let hud_spring = CASpring::new(0.35, 0.85);
    assert!(
        (hud_spring.omega_0() - 17.951958).abs() < tolerance,
        "HUD omega_0 expected 17.951958, got {}",
        hud_spring.omega_0()
    );
    assert!(
        (hud_spring.omega_d() - 9.456781).abs() < tolerance,
        "HUD omega_d expected 9.456781, got {}",
        hud_spring.omega_d()
    );
    assert_eq!(hud_spring.damping_ratio, 0.85);

    // Initial boundary conditions
    assert_eq!(hud_spring.evaluate(0.0), 0.0);
    assert_eq!(hud_spring.velocity(0.0), 0.0);

    // HUD settling duration
    let t_settle_hud = hud_spring.settling_duration();
    assert!(
        (t_settle_hud - 0.453).abs() < 0.002,
        "HUD t_settle expected ~0.453s, got {}",
        t_settle_hud
    );
    // At envelope decay boundary
    assert!(
        (hud_spring.evaluate(t_settle_hud) - 1.0).abs() <= 0.002,
        "HUD rest boundary deviation: {}",
        (hud_spring.evaluate(t_settle_hud) - 1.0).abs()
    );
    // Fully settled within 0.1% threshold at 0.5s
    assert!(
        (hud_spring.evaluate(0.50) - 1.0).abs() <= 0.001,
        "HUD rest tolerance at 0.5s: {}",
        (hud_spring.evaluate(0.50) - 1.0).abs()
    );
}

/// Test 3: Asserts compact HUD panel budget and zero-clipping guarantee (SPEC-001 Section 4.1).
#[test]
fn test_compact_hud_panel_budget_zero_clipping() {
    let tokens = MacDesignTokens::load_default();

    // Standard HUD panel dimensions
    let total_panel_height = tokens.popover_metrics().compact_panel_height.points();
    let header_height = tokens.control_metrics().text_field.regular.height.points();
    assert_eq!(total_panel_height, 63.0);
    assert_eq!(header_height, 22.0);

    // Available body budget
    let available_body_budget = total_panel_height - header_height;
    assert_eq!(available_body_budget, 41.0);

    // 2-row layout with two 18.0pt compact cells and 1.0pt vertical gap
    let cell_height = tokens
        .control_metrics()
        .text_field
        .compact_infobox
        .height
        .points();
    assert_eq!(cell_height, 18.0);

    let vertical_gap = 1.0;
    let claimed_body_height = cell_height + vertical_gap + cell_height;
    assert_eq!(claimed_body_height, 37.0);

    // Zero-clipping verification: claimed height must not exceed available budget
    assert!(
        claimed_body_height <= available_body_budget,
        "Body height {} exceeds available budget {}",
        claimed_body_height,
        available_body_budget
    );

    let safety_margin = available_body_budget - claimed_body_height;
    assert_eq!(safety_margin, 4.0);
    assert!(
        safety_margin > 0.0,
        "Zero-clipping guarantee requires strictly positive safety margin"
    );
}

/// Test 4: Tests subpixel snap criterion on Windows 150% fractional scaling (SPEC-001 Section 2.1 A).
#[test]
fn test_subpixel_snap_windows_150_percent() {
    let scale = 1.5; // Windows 150% scaling

    // Test specific boundary values
    let test_coords = [0.0, 0.25, 0.333, 0.5, 0.667, 1.0, 10.1, 10.4, 25.75, 41.333];

    for &x_logical in &test_coords {
        let x_snapped = subpixel_snap(x_logical, scale);
        let hardware_pixels = x_snapped * scale;
        let rounded_pixels = hardware_pixels.round();

        // Hardware pixel must land exactly on an integer pixel boundary
        assert!(
            (hardware_pixels - rounded_pixels).abs() < 1e-4,
            "Logical coord {} snapped to {} does not align with hardware integer pixel boundary at scale 1.5x (got {} px)",
            x_logical,
            x_snapped,
            hardware_pixels
        );
    }

    // Verify 2D point and Rect snapping
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

    // 1. Verify exact Apple HIG ground truth tracking table points
    let pt6_table = typo
        .tracking_table
        .get("pt_6")
        .expect("pt_6 entry missing")
        .tracking_pt;
    let pt12_table = typo
        .tracking_table
        .get("pt_12")
        .expect("pt_12 entry missing")
        .tracking_pt;
    let pt20_table = typo
        .tracking_table
        .get("pt_20")
        .expect("pt_20 entry missing")
        .tracking_pt;

    assert_eq!(pt6_table, 0.24, "HIG table pt_6 must be +0.24pt");
    assert_eq!(pt12_table, 0.00, "HIG table pt_12 must be neutral 0.00pt");
    assert_eq!(pt20_table, -0.45, "HIG table pt_20 must be -0.45pt");

    // 2. Validate continuous cubic polynomial against HIG optical behavior
    let track_6 = typo.tracking_for_size(6.0);
    let track_12 = typo.tracking_for_size(12.0);
    let track_20 = typo.tracking_for_size(20.0);

    // 6pt: Positive loose tracking for small text optical legibility
    assert!(
        track_6 > 0.15,
        "6pt tracking must be positive loose: got {}",
        track_6
    );
    assert!(
        (track_6 - 0.24).abs() < 0.05,
        "6pt tracking polynomial deviation too large: {}",
        track_6
    );

    // 12pt: Neutral zero-crossing baseline
    assert!(
        track_12.abs() < 0.02,
        "12pt tracking must be close to neutral 0.0pt: got {}",
        track_12
    );

    // 20pt: Display threshold negative tight tracking
    assert!(
        track_20 < -0.35,
        "20pt tracking must be negative display tight: got {}",
        track_20
    );
    assert!(
        (track_20 - (-0.45)).abs() < 0.05,
        "20pt tracking polynomial deviation too large: {}",
        track_20
    );

    // Monotonic decrease from micro to display sizing
    assert!(track_6 > track_12);
    assert!(track_12 > track_20);

    // Verify standalone sf_pro_tracking helper matches
    assert!((sf_pro_tracking(6.0) as f64 - track_6).abs() < 1e-4);
    assert!((sf_pro_tracking(12.0) as f64 - track_12).abs() < 1e-4);
    assert!((sf_pro_tracking(20.0) as f64 - track_20).abs() < 1e-4);
}

/// Test 6: Verifies 1-to-1 version synchronization across Cargo.toml, CHANGELOG.md, design_tokens.json, and all specs in specs/.
#[test]
fn test_version_synchronization_invariants() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // 1. Cargo.toml version
    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(
        cargo_toml.contains("version = \"0.3.0\""),
        "Cargo.toml must have version = \"0.3.0\""
    );

    // 2. Exported crate version constant
    assert_eq!(egui_macview::VERSION, "0.3.0");

    // 3. Design tokens version
    let tokens = MacDesignTokens::load_default();
    assert_eq!(tokens.version, "0.3.0");

    // 4. CHANGELOG.md strictly documents [0.3.0] - 2026-11-06, [0.2.0] - 2026-11-06 and [0.1.0] - 2026-11-05
    let changelog =
        fs::read_to_string(manifest_dir.join("CHANGELOG.md")).expect("read CHANGELOG.md");
    assert!(
        changelog.contains("## [0.3.0] - 2026-11-06"),
        "CHANGELOG.md must contain header '## [0.3.0] - 2026-11-06'"
    );
    assert!(
        changelog.contains("## [0.2.0] - 2026-11-06"),
        "CHANGELOG.md must contain header '## [0.2.0] - 2026-11-06'"
    );
    assert!(
        changelog.contains("## [0.1.0] - 2026-11-05"),
        "CHANGELOG.md must contain header '## [0.1.0] - 2026-11-05'"
    );

    // 5. Specs in specs/ have synchronized versions according to release roadmap:
    // - SPEC-001..SPEC-006 and SPEC-INDEX have Document Version 0.2.0
    // - SPEC-007 has Document Version 0.3.0 (Milestone 0.3.0 Released)
    // - SPEC-008 has Document Version 0.4.0-draft (Milestone 0.4.0)
    let specs_dir = manifest_dir.join("specs");
    let entries = fs::read_dir(&specs_dir).expect("read specs dir");
    let mut spec_count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            spec_count += 1;
            let content = fs::read_to_string(&path).expect("read spec file");
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            if file_name.starts_with("SPEC-007") || file_name.starts_with("SPEC-009") {
                assert!(
                    content.contains("**Document Version** | `0.3.0`"),
                    "{file_name} missing Document Version `0.3.0`"
                );
                assert!(
                    content.contains("Milestone 0.3.0"),
                    "{file_name} missing Target Milestone `Milestone 0.3.0`"
                );
            } else if file_name.starts_with("SPEC-008") {
                assert!(
                    content.contains("**Document Version** | `0.4.0-draft`"),
                    "{file_name} missing Document Version `0.4.0-draft`"
                );
                assert!(
                    content.contains("Milestone 0.4.0"),
                    "{file_name} missing Target Milestone `Milestone 0.4.0`"
                );
            } else {
                assert!(
                    content.contains("**Document Version** | `0.2.0`"),
                    "{file_name} missing Document Version `0.2.0`"
                );
            }

            assert!(
                content.contains("`0.1.0`") && content.contains("2026-11-05"),
                "{file_name} missing baseline 0.1.0 in Revision History"
            );
        }
    }
    assert!(
        spec_count >= 8,
        "Expected at least 8 specs, found {spec_count}"
    );
}
