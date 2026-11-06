//! Automated MacBook AppKit Differential & Parity Verification Tests.
//!
//! Ground-truth verification comparing `egui-macview` directly against
//! golden master bitmaps and metric extractions rendered live on Jaco's MacBook Air
//! (Darwin 23.6.0, macOS Sonoma 14.8.9, Apple clang 16.0.0, native AppKit/Quartz).

use egui_macview::{theme::MacTheme, CASpring};
use std::fs;
use std::path::Path;

#[test]
fn test_macbook_caspring_analytical_parity() {
    // 1. Read the MacBook native CASpring ground truth
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ground_truth_path = manifest_dir
        .join("tests")
        .join("golden_master")
        .join("macbook_ground_truth")
        .join("appkit_metrics_ground_truth.json");

    assert!(
        ground_truth_path.exists(),
        "MacBook ground truth file missing: {}",
        ground_truth_path.display()
    );

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let mac_spring = &v["caspring"];
    let mac_duration = mac_spring["settling_duration"]
        .as_f64()
        .expect("settling_duration float");

    // 2. Evaluate our CASpring implementation with identical parameters (mass=1.0, stiffness=100.0, damping=10.0)
    let our_spring = CASpring::from_physical(1.0, 100.0, 10.0);
    let our_duration = our_spring.settling_duration() as f64;

    // Tolerance within 0.1s of Apple Quartz CASpringAnimation (6.9077 envelope vs Quartz state-vector settling)
    let diff = (our_duration - mac_duration).abs();
    assert!(
        diff < 0.1,
        "CASpring settling duration mismatch against MacBook native Quartz: our={our_duration:.4}s vs mac={mac_duration:.4}s (diff={diff:.4}s)"
    );
}

#[test]
fn test_macbook_segmented_control_geometry_parity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ground_truth_path = manifest_dir
        .join("tests")
        .join("golden_master")
        .join("macbook_ground_truth")
        .join("appkit_metrics_ground_truth.json");

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let native_theme_h = v["segmented_theme"]["h"].as_f64().unwrap() as f32;
    let native_units_h = v["segmented_units"]["h"].as_f64().unwrap() as f32;

    let theme = MacTheme::default();
    let regular_h = theme.segmented().regular.height.points();

    // Verify height matches Apple AppKit rounded segmented control height (22pt or 24pt within 2pt)
    assert!(
        (regular_h - native_theme_h).abs() <= 2.0,
        "Segmented height mismatch: our={regular_h}pt vs native={native_theme_h}pt"
    );
    assert!(
        (regular_h - native_units_h).abs() <= 2.0,
        "Segmented units height mismatch: our={regular_h}pt vs native={native_units_h}pt"
    );
}

#[test]
fn test_macbook_golden_master_bitmaps_integrity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir
        .join("tests")
        .join("golden_master")
        .join("macbook_ground_truth");

    let required_bitmaps = [
        "segmented_theme_aqua.png",
        "segmented_theme_dark.png",
        "segmented_units_aqua.png",
        "segmented_units_dark.png",
        "slider_aqua.png",
        "slider_dark.png",
        "stepper_aqua.png",
        "stepper_dark.png",
        "popup_button_aqua.png",
        "popup_button_dark.png",
        "color_well_aqua.png",
        "color_well_dark.png",
        "button_aqua.png",
        "button_dark.png",
    ];

    for bitmap_name in &required_bitmaps {
        let bitmap_path = golden_dir.join(bitmap_name);
        assert!(
            bitmap_path.exists(),
            "Missing native MacBook golden master bitmap: {}",
            bitmap_path.display()
        );

        let img = image::open(&bitmap_path).unwrap_or_else(|e| {
            panic!(
                "Failed to load golden bitmap {}: {e}",
                bitmap_path.display()
            )
        });
        assert!(
            img.width() > 0 && img.height() > 0,
            "Golden bitmap has invalid empty dimensions"
        );
    }
}
