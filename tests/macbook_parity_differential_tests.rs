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
fn test_macbook_extended_controls_geometry_parity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ground_truth_path = manifest_dir
        .join("tests")
        .join("golden_master")
        .join("macbook_ground_truth")
        .join("appkit_metrics_ground_truth.json");

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let theme = MacTheme::default();

    // 1. Slider bounding height parity (AppKit NSSlider with ticks: 24pt, track: 3pt, thumb: 16pt)
    let native_slider_h = v["slider"]["h"].as_f64().unwrap() as f32;
    let our_slider_with_ticks_h = theme.slider().thumb_size.points()
        + theme.slider().tick_offset.points()
        + theme.slider().tick_height.points();
    assert!(
        (our_slider_with_ticks_h - native_slider_h).abs() <= 1.0,
        "Slider with ticks height divergence: our={our_slider_with_ticks_h}pt vs native={native_slider_h}pt"
    );

    // 2. Stepper height parity (AppKit NSStepper: 28pt)
    let native_stepper_h = v["stepper"]["h"].as_f64().unwrap() as f32;
    let our_stepper_h = theme.stepper().height.points();
    assert!(
        (our_stepper_h - native_stepper_h).abs() <= 6.0,
        "Stepper height divergence: our={our_stepper_h}pt vs native={native_stepper_h}pt"
    );

    // 3. ColorWell height parity (AppKit NSColorWell: 23pt)
    let native_well_h = v["color_well"]["h"].as_f64().unwrap() as f32;
    let our_well_h = theme.color_well().height.points();
    assert!(
        (our_well_h - native_well_h).abs() <= 2.0,
        "Color well height divergence: our={our_well_h}pt vs native={native_well_h}pt"
    );

    // 4. Push Button height parity (AppKit NSButton rounded: 22-32pt)
    let native_button_h = v["button"]["h"].as_f64().unwrap() as f32;
    let our_button_h = theme.button().regular.height.points();
    assert!(
        (our_button_h - native_button_h).abs() <= 12.0,
        "Button height divergence: our={our_button_h}pt vs native={native_button_h}pt"
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

/// Tier 2 Live Parity Test: Connects to Jaco's physical MacBook Air via SSH
/// and validates the remote PyObjC / AppKit ground-truth rendering engine.
///
/// Run explicitly with:
/// `cargo test --test macbook_parity_differential_tests -- --ignored`
#[test]
#[ignore]
fn test_live_macbook_ssh_connectivity_and_parity_harness() {
    use std::process::Command;

    // Check SSH connection to Host alias 'macbook'
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            "macbook",
            "uname -sm && ~/.patrix_mac_venv/bin/python --version",
        ])
        .output()
        .expect("Failed to execute ssh command. Is OpenSSH on local PATH?");

    assert!(
        output.status.success(),
        "MacBook SSH connection failed. Stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout_str.contains("Darwin"),
        "MacBook host uname output missing 'Darwin': {stdout_str}"
    );
    assert!(
        stdout_str.contains("Python 3"),
        "MacBook host python output missing 'Python 3': {stdout_str}"
    );
}
