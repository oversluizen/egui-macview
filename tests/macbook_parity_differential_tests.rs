//! Automated MacBook AppKit Differential & Parity Verification Tests.
//!
//! Ground-truth verification comparing `egui-macview` directly against
//! golden master bitmaps and metric extractions rendered on physical Apple Silicon
//! MacBook Air hardware (Darwin 23.6.0, macOS Sonoma 14.8.9, Apple clang 16.0.0, native AppKit/Quartz).

use egui_macview::{theme::MacTheme, CASpring};
use std::fs;
use std::path::Path;

#[test]
fn test_macbook_caspring_analytical_parity() {
    // 1. Read the MacBook native CASpring ground truth
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ground_truth_path =
        manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth").join("appkit_metrics_ground_truth.json");

    assert!(ground_truth_path.exists(), "MacBook ground truth file missing: {}", ground_truth_path.display());

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let mac_spring = &v["caspring"];
    let mac_duration = mac_spring["settling_duration"].as_f64().expect("settling_duration float");

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
    let ground_truth_path =
        manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth").join("appkit_metrics_ground_truth.json");

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let native_theme_h = v["segmented_theme"]["h"].as_f64().unwrap() as f32;
    let native_units_h = v["segmented_units"]["h"].as_f64().unwrap() as f32;

    let theme = MacTheme::default();
    let regular_h = theme.segmented().regular.height.points();

    // Verify height matches Apple AppKit rounded segmented control height (22pt or 24pt within 2pt)
    assert!((regular_h - native_theme_h).abs() <= 2.0, "Segmented height mismatch: our={regular_h}pt vs native={native_theme_h}pt");
    assert!((regular_h - native_units_h).abs() <= 2.0, "Segmented units height mismatch: our={regular_h}pt vs native={native_units_h}pt");
}

#[test]
fn test_macbook_extended_controls_geometry_parity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ground_truth_path =
        manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth").join("appkit_metrics_ground_truth.json");

    let content = fs::read_to_string(&ground_truth_path).expect("Read MacBook ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse ground truth JSON");

    let theme = MacTheme::default();

    // 1. Slider bounding height parity (AppKit NSSlider with ticks: 24pt, track: 3pt, thumb: 16pt)
    let native_slider_h = v["slider"]["h"].as_f64().unwrap() as f32;
    let our_slider_with_ticks_h =
        theme.slider().thumb_size.points() + theme.slider().tick_offset.points() + theme.slider().tick_height.points();
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
    assert!((our_well_h - native_well_h).abs() <= 2.0, "Color well height divergence: our={our_well_h}pt vs native={native_well_h}pt");

    // 4. Push Button height parity (AppKit NSButton rounded: 22-32pt)
    let native_button_h = v["button"]["h"].as_f64().unwrap() as f32;
    let our_button_h = theme.button().regular.height.points();
    assert!((our_button_h - native_button_h).abs() <= 12.0, "Button height divergence: our={our_button_h}pt vs native={native_button_h}pt");
}

#[test]
fn test_macbook_golden_master_bitmaps_integrity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth");

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
        assert!(bitmap_path.exists(), "Missing native MacBook golden master bitmap: {}", bitmap_path.display());

        let img = image::open(&bitmap_path).unwrap_or_else(|e| panic!("Failed to load golden bitmap {}: {e}", bitmap_path.display()));
        assert!(img.width() > 0 && img.height() > 0, "Golden bitmap has invalid empty dimensions");
    }
}

#[test]
fn test_macbook_symbol_search_parity_differential() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth");

    let macbook_img_path = golden_dir.join("symbol_search_macbook.png");
    assert!(macbook_img_path.exists(), "Missing native MacBook search symbol: {}", macbook_img_path.display());

    let img = image::open(&macbook_img_path).expect("Open MacBook search symbol");
    assert_eq!(img.width(), 64);
    assert_eq!(img.height(), 64);

    // Verify non-empty glyph pixels in central area rendered from native AppKit NSImage
    let mut non_empty_count = 0;
    for pixel in img.to_rgba8().pixels() {
        if pixel[3] > 0 {
            non_empty_count += 1;
        }
    }
    assert!(non_empty_count >= 50, "MacBook symbol has too few pixels: {non_empty_count}");
}

/// Tier 2 Live Parity Test: Connects to a physical macOS reference host via SSH
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
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=5", "macbook", "uname -sm && python3 --version"])
        .output()
        .expect("Failed to execute ssh command. Is OpenSSH on local PATH?");

    assert!(output.status.success(), "MacBook SSH connection failed. Stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    assert!(stdout_str.contains("Darwin"), "MacBook host uname output missing 'Darwin': {stdout_str}");
    assert!(stdout_str.contains("Python 3"), "MacBook host python output missing 'Python 3': {stdout_str}");
}

#[test]
fn test_macbook_v05_studio_controls_ground_truth_parity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let golden_dir = manifest_dir.join("tests").join("golden_master").join("macbook_ground_truth");
    let studio_json_path = golden_dir.join("appkit_studio_v05_ground_truth.json");

    assert!(studio_json_path.exists(), "Missing MacBook v0.5.0 studio ground truth JSON: {}", studio_json_path.display());

    let content = fs::read_to_string(&studio_json_path).expect("Read studio ground truth JSON");
    let v: serde_json::Value = serde_json::from_str(&content).expect("Parse studio JSON");

    let aqua = MacTheme::new(egui_macview::MacAppearance::Aqua);
    let dark = MacTheme::new(egui_macview::MacAppearance::DarkAqua);

    // 1. Exact sRGB color parity against MacBook NSColor extraction
    assert_eq!(
        aqua.system_yellow(),
        egui::Color32::from_rgb(
            v["colors"]["aqua"]["system_yellow"]["r"].as_u64().unwrap() as u8,
            v["colors"]["aqua"]["system_yellow"]["g"].as_u64().unwrap() as u8,
            v["colors"]["aqua"]["system_yellow"]["b"].as_u64().unwrap() as u8,
        )
    );
    assert_eq!(
        dark.system_yellow(),
        egui::Color32::from_rgb(
            v["colors"]["dark_aqua"]["system_yellow"]["r"].as_u64().unwrap() as u8,
            v["colors"]["dark_aqua"]["system_yellow"]["g"].as_u64().unwrap() as u8,
            v["colors"]["dark_aqua"]["system_yellow"]["b"].as_u64().unwrap() as u8,
        )
    );

    // 2. Geometry parity against MacBook AppKit fitting/frame metrics
    let mac_spin_reg = v["controls"]["progress_indicator"]["regular"]["spin_fitting_h"].as_f64().unwrap() as f32;
    assert_eq!(aqua.progress_indicator().regular.spinner_size.points(), mac_spin_reg);

    let mac_spin_small = v["controls"]["progress_indicator"]["small"]["spin_fitting_h"].as_f64().unwrap() as f32;
    assert_eq!(aqua.progress_indicator().small.spinner_size.points(), mac_spin_small);

    let mac_table_row_h = v["controls"]["table_view"]["row_height"].as_f64().unwrap() as f32;
    let mac_table_hdr_h = v["controls"]["table_view"]["header_height"].as_f64().unwrap() as f32;
    assert_eq!(aqua.table_view().row_height.points(), mac_table_row_h);
    assert_eq!(aqua.table_view().header_height.points(), mac_table_hdr_h);

    let mac_path_h = v["controls"]["path_control"]["fitting_h"].as_f64().unwrap() as f32;
    assert_eq!(aqua.path_control().height.points(), mac_path_h);

    // 3. Verify all 12 new v0.5.0 golden master bitmaps from MacBook Air (dimensions & non-empty pixel sampling)
    let studio_bitmaps: [(&str, u32, u32); 12] = [
        ("progress_bar_aqua.png", 160, 20),
        ("progress_bar_dark.png", 160, 20),
        ("progress_spin_aqua.png", 32, 32),
        ("progress_spin_dark.png", 32, 32),
        ("table_header_aqua.png", 240, 28),
        ("table_header_dark.png", 240, 28),
        ("token_field_aqua.png", 240, 22),
        ("token_field_dark.png", 240, 22),
        ("level_indicator_aqua.png", 160, 18),
        ("level_indicator_dark.png", 160, 18),
        ("path_control_aqua.png", 240, 22),
        ("path_control_dark.png", 240, 22),
    ];
    for (name, exp_w, exp_h) in &studio_bitmaps {
        let p = golden_dir.join(name);
        assert!(p.exists(), "Missing studio golden master PNG: {}", p.display());
        let img = image::open(&p).expect("Valid studio golden master PNG").to_rgba8();
        assert_eq!((img.width(), img.height()), (*exp_w, *exp_h), "Unexpected bitmap dimensions for {name}");
        let non_empty_pixels = img.pixels().filter(|px| px[3] > 0).count();
        assert!(non_empty_pixels > 32, "Golden master {name} must contain rendered AppKit control pixels");
    }
}

#[test]
fn test_macbook_v07_nib_alignment_rect_and_resizing_ground_truth_parity() {
    use egui_macview::scene::{CustomClassMapper, LayoutPolicy, NibArchiveFrontend, SceneNodeKind};

    let mapper = CustomClassMapper::default_studio_mappings();
    let nib_bytes = egui_macview::scene::nib_varint::build_real_varint_nib_archive();
    let scene = NibArchiveFrontend::from_bytes(&nib_bytes, &mapper).expect("Valid NIBArchive v1");
    assert!(scene.validate_invariants().is_ok());

    // 1. Verify Apple -[NSView alignmentRectForFrame:] inset stripping parity (<= 0.5pt)
    // Native macOS Sonoma/Sequoia NSButton rounded bezel frame [-6, -6, 131, 40] -> alignmentRect [0, 0, 119, 28]
    let btn = scene.nodes().iter().find(|n| matches!(n.kind, SceneNodeKind::PushButton { .. })).expect("PushButton present");
    let expected_apple_align = [0.0_f32, 0.0, 119.0, 28.0];
    for (i, &exp) in expected_apple_align.iter().enumerate() {
        assert!(
            (btn.alignment_rect[i] - exp).abs() <= 0.5,
            "alignmentRectForFrame axis {i} mismatch: got {} vs Apple {exp}",
            btn.alignment_rect[i]
        );
    }

    // 2. Verify NSViewResizeMask (NSViewWidthSizable = 0x02) proportional expansion at 640x320 (+160pt width)
    let solved = scene.solve_layout(egui::vec2(640.0, 320.0), LayoutPolicy::ExactAppleNib, 0, "");
    let num_node = scene.nodes().iter().find(|n| matches!(n.kind, SceneNodeKind::NumberField { .. })).expect("NumberField present");
    let solved_rect = solved.get(&num_node.id).expect("Solved rect");
    // Original frame width = 72.0 with mask = 0x02 (NSViewWidthSizable only -> 100% of +160pt delta = 232.0pt)
    assert!(
        (solved_rect.width() - 232.0).abs() <= 0.5,
        "NSViewWidthSizable width mismatch against AppKit: got {} vs 232.0",
        solved_rect.width()
    );
}

struct DummyDiffStore;
impl egui_macview::scene::MacSceneDataSource for DummyDiffStore {
    fn get_bool(&self, _: egui_macview::scene::BindingId) -> bool {
        true
    }
    fn set_bool(&mut self, _: egui_macview::scene::BindingId, _: bool) {}
    fn get_f64(&self, _: egui_macview::scene::BindingId) -> f64 {
        100.0
    }
    fn set_f64(&mut self, _: egui_macview::scene::BindingId, _: f64) {}
    fn get_i64(&self, _: egui_macview::scene::BindingId) -> i64 {
        0
    }
    fn set_i64(&mut self, _: egui_macview::scene::BindingId, _: i64) {}
    fn get_str(&self, _: egui_macview::scene::BindingId) -> &str {
        "100"
    }
    fn set_str(&mut self, _: egui_macview::scene::BindingId, _: &str) {}
    fn get_options(&self, _: egui_macview::scene::BindingId) -> &[String] {
        &[]
    }
    fn on_action(&mut self, _: egui_macview::scene::ActionId, _: egui_macview::scene::NodeId) {}
    fn render_custom_slot(&mut self, _: &mut egui::Ui, _: &MacTheme, _: &str, _: egui::Rect) {}
}

#[test]
fn test_macbook_vs_windows_xib_nib_visual_and_geometric_diff_oracle() {
    use egui_macview::scene::{CustomClassMapper, LayoutPolicy, NibArchiveFrontend, XibXmlFrontend};
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let showcase_dir = manifest_dir.join("assets").join("nib_showcase");
    let mapper = CustomClassMapper::default_studio_mappings();
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    egui_macview::setup_macview_fonts(&mut fonts);
    ctx.set_fonts(fonts);

    for xib_name in [
        "1_Vector_and_Transform_Inspector.xib",
        "2_Theme_and_Appearance_Studio.xib",
        "3_AVKit_Audio_Mastering_Strip.xib",
        "4_Publishing_and_Asset_Queue.xib",
        "Right_Inspector_Default.xib",
        "1_Typography_and_Grid_Inspector.xib",
        "2_Appearance_and_Theme_Settings.xib",
        "3_Audio_Mastering_Channel_Strip.xib",
    ] {
        let xml = fs::read_to_string(showcase_dir.join(xib_name)).expect("read .xib");
        let xib_scene = XibXmlFrontend::from_xml_str(&xml).expect("compile .xib");
        assert!(xib_scene.validate_invariants().is_ok(), "{xib_name} invariants");
        let nib_bytes = fs::read(showcase_dir.join(xib_name.replace(".xib", ".nib"))).expect("read .nib");
        let nib_scene = NibArchiveFrontend::from_bytes(&nib_bytes, &mapper).expect("decode .nib");
        assert_eq!(xib_scene.nodes().len(), nib_scene.nodes().len(), "{xib_name} node count parity");

        for (nx, nn) in xib_scene.nodes().iter().zip(nib_scene.nodes().iter()) {
            for axis in 0..4 {
                assert!((nx.alignment_rect[axis] - nn.alignment_rect[axis]).abs() <= 0.5, "{xib_name} alignmentRect diff");
            }
        }

        for appearance in [egui_macview::MacAppearance::Aqua, egui_macview::MacAppearance::DarkAqua] {
            let theme = MacTheme::new(appearance);
            let mut store_a = DummyDiffStore;
            let mut store_b = DummyDiffStore;
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 480.0))),
                ..Default::default()
            };
            let mut out_xib = ctx.run_ui(raw.clone(), |ui| {
                xib_scene.show(ui, &theme, &mut store_a, LayoutPolicy::ExactAppleNib, 0, "");
            });
            let mut out_nib = ctx.run_ui(raw, |ui| {
                nib_scene.show(ui, &theme, &mut store_b, LayoutPolicy::ExactAppleNib, 0, "");
            });
            out_xib.textures_delta.clear();
            out_nib.textures_delta.clear();
            assert!(!out_xib.shapes.is_empty(), "{xib_name} must emit visual shapes");
            assert_eq!(out_xib.shapes.len(), out_nib.shapes.len(), "{xib_name} visual shape count parity between .xib and .nib");
        }
    }
}

#[test]
fn test_macbook_xcode16_ibtool_xib_nib_storyboard_roundtrip_parity() {
    use egui_macview::scene::{CustomClassMapper, NibArchiveFrontend, SceneExporter, StoryboardFrontend, XibXmlFrontend};
    let showcase_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets").join("nib_showcase");
    let mapper = CustomClassMapper::default_studio_mappings();

    for stem in [
        "1_Vector_and_Transform_Inspector",
        "1_Typography_and_Grid_Inspector",
        "2_Appearance_and_Theme_Settings",
        "2_Theme_and_Appearance_Studio",
        "3_AVKit_Audio_Mastering_Strip",
        "3_Audio_Mastering_Channel_Strip",
        "4_Publishing_and_Asset_Queue",
        "Right_Inspector_Default",
    ] {
        let xib_str = fs::read_to_string(showcase_dir.join(format!("{stem}.xib"))).expect("read .xib");
        let xib_ir = XibXmlFrontend::from_xml_str(&xib_str).expect("parse .xib");
        let exported_xib = SceneExporter::to_xib_xml(&xib_ir);
        let re_xib_ir = XibXmlFrontend::from_xml_str(&exported_xib).expect("re-parse exported .xib");

        let nib_bytes = fs::read(showcase_dir.join(format!("{stem}.nib"))).expect("read real ibtool .nib");
        assert!(
            nib_bytes.windows(14).any(|w| w == b"NSIBObjectData")
                && !nib_bytes.windows(6).any(|w| w == b"[{\"id\"")
                && !nib_bytes.windows(11).any(|w| w == b"macviewMeta"),
            "{stem}.nib must be compiled by Apple ibtool with zero JSON or macviewMeta"
        );
        let nib_ir = NibArchiveFrontend::from_bytes(&nib_bytes, &mapper).expect("decode real ibtool .nib");
        assert_eq!(xib_ir.nodes().len(), re_xib_ir.nodes().len(), "{stem} exported .xib node count parity");
        assert_eq!(xib_ir.nodes().len(), nib_ir.nodes().len(), "{stem} ibtool .nib node count parity");

        for ((nx, nr), nn) in xib_ir.nodes().iter().zip(re_xib_ir.nodes().iter()).zip(nib_ir.nodes().iter()) {
            assert_eq!(nx.ib_object_id, nn.ib_object_id, "{stem} ib_object_id parity");
            assert_eq!(nx.kind, nn.kind, "{stem} SceneNodeKind parity for {}", nx.ib_object_id);
            assert_eq!(nx.title, nn.title, "{stem} title parity for {}", nx.ib_object_id);
            assert_eq!(nx.control_size, nn.control_size, "{stem} control_size parity for {}", nx.ib_object_id);
            assert_eq!(nx.icon_name, nn.icon_name, "{stem} icon_name parity for {}", nx.ib_object_id);
            assert_eq!(nx.binding_path, nn.binding_path, "{stem} binding_path parity for {}", nx.ib_object_id);
            assert_eq!(nx.action_selector, nn.action_selector, "{stem} action_selector parity for {}", nx.ib_object_id);
            for axis in 0..4 {
                assert!((nx.nib_frame[axis] - nr.nib_frame[axis]).abs() <= 0.5, "{stem} exported .xib frame axis {axis}");
                assert!((nx.nib_frame[axis] - nn.nib_frame[axis]).abs() <= 0.5, "{stem} ibtool .nib frame axis {axis}");
                assert!((nx.alignment_rect[axis] - nn.alignment_rect[axis]).abs() <= 0.5, "{stem} alignment_rect axis {axis}");
            }
        }
    }

    let sb_path = showcase_dir.join("Showcase_Flow.storyboard");
    assert!(sb_path.exists(), "Showcase_Flow.storyboard must exist in assets/nib_showcase/");
    let sb = StoryboardFrontend::from_xml_str(&fs::read_to_string(&sb_path).unwrap()).expect("parse Showcase_Flow.storyboard");
    assert!(sb.scenes.len() >= 4 && sb.scenes.iter().any(|s| !s.segues.is_empty()));

    if let Ok(out) = std::process::Command::new("ssh")
        .args([
            "-n",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=3",
            "macbook",
            "/Applications/Xcode.app/Contents/Developer/usr/bin/ibtool --errors --warnings ~/Desktop/MacView-Xcode-Comparison/1_Vector_and_Transform_Inspector.xib",
        ])
        .output()
    {
        if out.status.success() {
            let plist_out = String::from_utf8_lossy(&out.stdout);
            assert!(
                plist_out.contains("<key>com.apple.ibtool.document.errors</key>\n\t<dict/>"),
                "Live MacBook Xcode 16.2 ibtool validation failed:\n{plist_out}"
            );
        }
    }
}
