//! Automated Architecture Guard Suite (Golden Master Fitness Functions).
//!
//! Normative reference: Automated Architecture Fitness Guards (Zero Hardcoding, Zero Token Cheating,
//! Domain-Agnostic Separation, Documentation Completeness, File Size Ceilings, Windows GUI Subsystem
//! & Safe `eframe` Startup, Zero Dead Code & Zero Unused Dependencies, and State-Invariant Roundtrip Verification).

mod guard_helpers;

use guard_helpers::*;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const UI_COMPONENT_DIRS: &[&str] = &["src/controls", "src/layout", "src/surfaces", "src/navigation", "src/feedback"];
const EXTRA_GUARDED_FILES: &[&str] = &["src/theme/mod.rs", "src/theme/accessors.rs", "src/theme/pro_accessors.rs"];

#[test]
fn test_guard_zero_magic_numbers_and_colors_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut magic_v, mut color_v) = (Vec::new(), Vec::new());
    for dir_rel in UI_COMPONENT_DIRS {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
            magic_v.extend(check_magic_numbers_in_source(&rel, &fs::read_to_string(&file).expect("read source")));
        }
    }
    for file_rel in EXTRA_GUARDED_FILES {
        magic_v.extend(check_magic_numbers_in_source(file_rel, &fs::read_to_string(manifest_dir.join(file_rel)).expect("read source")));
    }
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        if rel != "src/tokens/dtcg.rs" && rel != "src/theme/contrast.rs" {
            color_v.extend(check_hardcoded_colors_in_source(&rel, &fs::read_to_string(&file).expect("read source")));
        }
    }
    assert!(magic_v.is_empty(), "Magic Number Guard:\n{magic_v:#?}");
    assert!(color_v.is_empty(), "Hardcoded Color Guard:\n{color_v:#?}");
}

#[test]
fn test_guard_zero_token_arithmetic_and_domain_purity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut math_v, mut domain_v, mut salt_v) = (Vec::new(), Vec::new(), Vec::new());
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
        let content = fs::read_to_string(&file).expect("read source");
        math_v.extend(check_token_arithmetic_cheating(&rel, &content));
        domain_v.extend(check_domain_leakage_in_source(&rel, &content));
        salt_v.extend(check_uibuilder_id_salt_in_source(&rel, &content));
    }
    for file in collect_rs_files(&manifest_dir.join("examples")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
        let content = fs::read_to_string(&file).expect("read source");
        domain_v.extend(check_domain_leakage_in_source(&rel, &content));
        salt_v.extend(check_uibuilder_id_salt_in_source(&rel, &content));
    }
    assert!(math_v.is_empty(), "Token Math Guard:\n{math_v:#?}");
    assert!(domain_v.is_empty(), "Domain Purity Guard:\n{domain_v:#?}");
    assert!(salt_v.is_empty(), "UiBuilder id_salt Guard:\n{salt_v:#?}");
}

#[test]
fn test_guard_token_catalog_100_percent_schema_synchronization() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let raw_json = include_str!("../tokens/design_tokens.json");
    let raw_val: serde_json::Value = serde_json::from_str(raw_json).expect("Valid JSON");
    let typed_tokens = egui_macview::tokens::MacDesignTokens::load_default();
    let reserialized_val = std::thread::Builder::new().stack_size(8 * 1024 * 1024)
        .spawn(move || serde_json::to_value(&typed_tokens).expect("Serialize MacDesignTokens"))
        .expect("spawn").join().expect("join");
    let (mut raw_leaves, mut typed_leaves) = (BTreeSet::new(), BTreeSet::new());
    collect_json_leaf_paths(&raw_val, String::new(), false, &mut raw_leaves);
    collect_json_leaf_paths(&reserialized_val, String::new(), false, &mut typed_leaves);
    assert!(raw_leaves.difference(&typed_leaves).count() == 0 && typed_leaves.difference(&raw_leaves).count() == 0);
    for file in collect_rs_files(&manifest_dir.join("src/tokens")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        if rel != "src/tokens/dtcg.rs" {
            let stripped = strip_comments_and_strings(&fs::read_to_string(&file).expect("read token file"));
            assert!(!stripped.contains("DimensionValue::from_pt(") && !stripped.contains("ColorValue::new("), "{rel}");
        }
    }
}

#[test]
fn test_guard_documentation_and_file_size_ceilings() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap().contains("#![deny(missing_docs)]"));
    assert!(!fs::read_to_string(manifest_dir.join("README.md")).unwrap().contains("](src/"));
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let content = fs::read_to_string(&file).expect("read source");
        assert!(content.lines().take(5).any(|l| l.trim_start().starts_with("//!")));
    }
    let mut violations = Vec::new();
    for dir_rel in &["src", "tests", "examples"] {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
            let content = fs::read_to_string(&file).expect("read source");
            let (physical, esloc) = count_lines_and_esloc(&content);
            if physical >= MAX_PHYSICAL_LINES || esloc >= MAX_ESLOC {
                violations.push(format!("{rel} exceeds ceiling: physical={physical} (<{MAX_PHYSICAL_LINES}), eSLOC={esloc} (<{MAX_ESLOC})"));
            }
            for (line_idx, line) in content.lines().enumerate() {
                let code_part = line.split("//").next().unwrap_or("").trim();
                if !code_part.contains('"') && !code_part.ends_with('\\') && code_part.replace("; 2]", "]").replace("; 3]", "]").replace("; 4]", "]").matches(';').count() >= 3 {
                    violations.push(format!("{rel}:{} -> single-line statement cramming (>=3 semicolons)", line_idx + 1));
                }
            }
        }
    }
    assert!(violations.is_empty(), "Ceiling Guard:\n{violations:#?}");
}

#[test]
fn test_guard_zero_cross_component_token_borrowing_and_self_check() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let methods = [
        "box_view", "button", "checkbox", "collection_view", "color_well", "combo_box",
        "date_picker", "disclosure_group", "help_button", "inline_action_button",
        "level_indicator", "origin_matrix", "outline_view", "path_control", "player_view",
        "popover_metrics", "popup_button", "progress_indicator", "rule_editor", "scroll_view",
        "search_field", "segmented", "slider", "split_view", "stack_view", "stepper", "tab_view",
        "table_action_bar", "table_view", "tag_color_picker", "thumbnail", "toggle", "token_field",
        "trim_scrubber",
    ];
    for file_path in collect_rs_files(&manifest_dir.join("src/controls")) {
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let pname = file_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("");
        if fname == "mod.rs" && pname == "controls" {
            continue;
        }
        let allowed: &[&str] = match (pname, fname) {
            (_, "box_view.rs") => &["box_view"], (_, "button_cell.rs") => &["button"],
            (_, "checkbox.rs") => &["checkbox"], (_, "circular_slider.rs") | (_, "slider.rs") => &["slider"],
            (_, "collection_view.rs") => &["collection_view"], (_, "color_well.rs") => &["color_well"],
            (_, "combo_box.rs") | ("combo_box", _) => &["combo_box"], (_, "date_picker.rs") | ("date_picker", _) => &["date_picker"],
            (_, "disclosure_group.rs") | ("disclosure_group", _) => &["disclosure_group"], (_, "help_button.rs") => &["help_button", "inline_action_button"],
            (_, "level_indicator.rs") | (_, "rating_indicator.rs") => &["level_indicator"], (_, "origin_matrix.rs") => &["origin_matrix"],
            (_, "outline_view.rs") | ("outline_view", _) => &["outline_view", "thumbnail"], (_, "path_control.rs") => &["path_control"],
            (_, "player_view.rs") | ("player_view", _) => &["player_view"], (_, "popup_button.rs") | ("popup_button", _) => &["popup_button"],
            (_, "progress_indicator.rs") => &["progress_indicator"], (_, "rule_editor.rs") => &["rule_editor"],
            (_, "search_field.rs") => &["search_field"], (_, "segmented.rs") => &["segmented"],
            (_, "stepper.rs") => &["stepper"], (_, "tab_view.rs") => &["tab_view"],
            (_, "table_action_bar.rs") => &["table_action_bar"], ("table_view", _) => &["table_view"],
            (_, "tag_color_picker.rs") => &["tag_color_picker"], (_, "thumbnail_cell.rs") => &["thumbnail"],
            (_, "toggle.rs") => &["toggle"], (_, "token_field.rs") => &["token_field"],
            ("trim_scrubber", _) => &["trim_scrubber"], _ => &[],
        };
        let stripped = strip_comments_and_strings(&fs::read_to_string(&file_path).unwrap());
        for &m in &methods {
            if !allowed.contains(&m) {
                assert!(!stripped.contains(&format!("theme.{m}(")), "{fname} calls theme.{m}(");
            }
        }
    }
    assert!(!check_magic_numbers_in_source("bad.rs", "let h = 18.0;").is_empty());
    assert!(check_magic_numbers_in_source("ok.rs", "let mid = w * 0.5;").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = Color32::WHITE;").is_empty());
    assert!(!check_token_arithmetic_cheating("bad.rs", "let w = theme.width.points() + 4.0;").is_empty());
    assert!(!check_domain_leakage_in_source("bad.rs", "struct PedigreeNode;").is_empty());
    assert!(!check_uibuilder_id_salt_in_source("bad.rs", "ui.new_child(UiBuilder::new().max_rect(r));").is_empty());
    assert!(check_uibuilder_id_salt_in_source("ok.rs", "ui.new_child(UiBuilder::new().id_salt(\"s\").max_rect(r));").is_empty());
}

#[test]
fn test_guard_windows_gui_subsystem_and_safe_eframe_startup() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut violations, mut checked_entrypoints) = (Vec::new(), 0usize);
    for dir_rel in &["examples", "src/bin"] {
        let dir_path = manifest_dir.join(dir_rel);
        if !dir_path.exists() { continue; }
        for file in collect_rs_files(&dir_path) {
            let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
            let content = fs::read_to_string(&file).expect("read entrypoint candidate");
            if strip_comments_and_strings(&content).contains("fn main") { checked_entrypoints += 1; }
            violations.extend(check_gui_entrypoint_safety(&rel, &content));
        }
    }
    assert!(checked_entrypoints >= 3, "Expected at least 3 GUI entrypoints in examples/");
    assert!(violations.is_empty(), "Windows GUI Subsystem & Safe eframe Startup Guard:\n{violations:#?}");
    let bad_gui = "fn main() { let _ = eframe::run_native(\"app\", eframe::NativeOptions::default(), Box::new(|_| unreachable!())); }";
    assert!(check_gui_entrypoint_safety("bad_example.rs", bad_gui).len() >= 5);
}

#[test]
fn test_guard_zero_dead_code_and_zero_unused_dependencies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut dead_code_violations, mut combined_src) = (Vec::new(), String::new());
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
        let content = fs::read_to_string(&file).expect("read src file");
        dead_code_violations.extend(check_zero_dead_code_in_source(&rel, &content));
        combined_src.push_str(&strip_comments_and_strings(&content));
        combined_src.push('\n');
    }
    assert!(dead_code_violations.is_empty(), "Zero Dead Code Guard:\n{dead_code_violations:#?}");
    assert!(!check_zero_dead_code_in_source("src/bad.rs", "#[allow(dead_code)]\nstruct Ghost;").is_empty());

    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    let deps = extract_cargo_dependencies(&cargo_toml);
    assert!(!deps.is_empty());
    let mut unused_deps = Vec::new();
    for dep in &deps {
        let used = combined_src.contains(&format!("{dep}::"))
            || combined_src.contains(&format!("use {dep}"))
            || (dep == "serde" && (combined_src.contains("Serialize") || combined_src.contains("Deserialize")));
        if !used { unused_deps.push(dep.clone()); }
    }
    assert!(unused_deps.is_empty(), "Zero Unused Dependencies Guard:\n{unused_deps:#?}");

    let undo_src = fs::read_to_string(manifest_dir.join("src/state/undo.rs")).expect("read undo.rs");
    assert!(undo_src.contains("pub fn validate_invariants("));
    let hygiene_test = fs::read_to_string(manifest_dir.join("tests/runtime_hygiene_and_invariants_tests.rs")).expect("read hygiene test");
    assert!(hygiene_test.contains("validate_invariants()") && hygiene_test.contains(".undo(") && hygiene_test.contains(".redo(") && hygiene_test.contains("cold_start") && hygiene_test.contains("repaint_delay"));
}

#[test]
fn test_guard_zero_vault_leakage_and_de_ai_ification() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();
    for dir in &["src", "tests", "specs", "examples"] {
        for file in collect_rs_files(&manifest_dir.join(dir)) {
            let rel = file.strip_prefix(manifest_dir).unwrap_or(&file).to_string_lossy();
            let content = fs::read_to_string(&file).expect("read file");
            violations.extend(check_vault_leakage_and_dutch_in_text(&rel, &content));
        }
    }
    for dir_path in [manifest_dir.to_path_buf(), manifest_dir.join("specs"), manifest_dir.join("tokens")] {
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && matches!(path.extension().and_then(|s| s.to_str()), Some("md" | "toml" | "json" | "cmd")) {
                    let rel = path.strip_prefix(manifest_dir).unwrap_or(&path).to_string_lossy();
                    let content = fs::read_to_string(&path).expect("read file");
                    violations.extend(check_vault_leakage_and_dutch_in_text(&rel, &content));
                }
            }
        }
    }
    assert!(!check_vault_leakage_and_dutch_in_text("dutch_sample.rs", &format!("// {}{} {}{}", "vo", "or", "kn", "op")).is_empty());
    assert!(violations.is_empty(), "Vault Leakage & Non-English Guard:\n{violations:#?}");
}

#[test]
fn test_guard_open_source_fonts_only_zero_proprietary_apple_fonts() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fonts_dir = manifest_dir.join("assets").join("fonts");
    let allowed_font_files = ["CupertinoIcons.ttf", "CupertinoIcons-LICENSE.txt", "Inter-Regular.ttf", "Inter-SemiBold.ttf", "Inter-OFL.txt"];
    let mut found_files = Vec::new();
    if let Ok(entries) = fs::read_dir(&fonts_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                found_files.push(name.to_string());
                assert!(allowed_font_files.contains(&name), "Forbidden font asset: {name}");
            }
        }
    }
    for required in &allowed_font_files { assert!(found_files.iter().any(|f| f == required)); }
    assert!(!fs::read_to_string(fonts_dir.join("Inter-OFL.txt")).unwrap().trim().is_empty());
    assert!(!fs::read_to_string(fonts_dir.join("CupertinoIcons-LICENSE.txt")).unwrap().trim().is_empty());
    let root_license = fs::read_to_string(manifest_dir.join("LICENSE")).unwrap();
    for clause in ["WITHOUT WARRANTY OF ANY KIND", "IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE", "SIL OPEN FONT LICENSE", "The Flutter Authors"] {
        assert!(root_license.contains(clause));
    }
    let readme = fs::read_to_string(manifest_dir.join("README.md")).unwrap();
    assert!(readme.contains("not affiliated with, endorsed by, or sponsored by Apple Inc."));
    let forbidden = ["SFNS.ttf", "SFNS.otf", "SFPro.ttf", "SFCompact.ttf", "Menlo.ttc", "LucidaGrande.ttc", "Apple Symbols.ttf"];
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let content = fs::read_to_string(&file).unwrap();
        for bad in forbidden { assert!(!content.contains(bad)); }
    }
}

#[test]
fn test_guard_surface_aware_vibrancy_anchoring() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let contrast_src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join("src/theme/contrast.rs")).unwrap());
    for req in ["pub fn plus_darker(", "pub fn plus_lighter(", "pub fn ensure_wcag_contrast(", "pub fn register_surface_rect(", "pub fn active_surface_at(", "pub fn active_surface(", "pub fn label_on(", "pub fn secondary_label_on(", "pub fn tertiary_label_on("] {
        assert!(contrast_src.contains(req), "src/theme/contrast.rs must expose '{req}'");
    }
    for bad in ["ACTIVE_SURFACE_OVERRIDE", "SurfaceScope", "with_surface_scope", "entry_panel_fill", "entry_seq", "next_seq"] {
        assert!(!contrast_src.contains(bad), "src/theme/contrast.rs must not contain '{bad}'");
    }
    for container_rel in ["src/controls/box_view.rs", "src/surfaces/card.rs", "src/surfaces/popover.rs"] {
        let src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join(container_rel)).unwrap());
        assert!(src.contains("visuals_mut().panel_fill") && src.contains("register_surface_rect_slot(") && !src.contains("with_surface_scope("), "{container_rel}");
    }
    for control_rel in ["src/controls/label.rs", "src/controls/checkbox.rs", "src/controls/disclosure_group/mod.rs", "src/controls/help_button.rs", "src/layout/stack_view.rs"] {
        let src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join(control_rel)).unwrap());
        assert!((src.contains("active_surface_at(") || src.contains("active_surface(")) && (src.contains("label_on(") || src.contains("secondary_label_on(") || src.contains("tertiary_label_on(") || src.contains("resolved_color_on(")), "{control_rel}");
    }
    let accessors_src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join("src/theme/accessors.rs")).unwrap());
    assert!(accessors_src.contains("pub fn baseline_text_y(") && accessors_src.contains("pub fn clamp_nib_visual_rect("));
    for bc in ["src/controls/label.rs", "src/controls/input_cell/editing.rs", "src/controls/popup_button/mod.rs", "src/controls/combo_box/mod.rs", "src/controls/checkbox.rs"] {
        let src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join(bc)).unwrap());
        assert!(src.contains("baseline_text_y(") && !src.contains("baseline_from_top - ascender") && !src.contains("first_baseline_offset_from_top.points() - ascender"), "{bc}");
    }
    for cc in ["src/controls/popup_button/mod.rs", "src/controls/combo_box/mod.rs", "src/controls/help_button.rs"] {
        let src = strip_comments_and_strings(&fs::read_to_string(manifest_dir.join(cc)).unwrap());
        assert!(src.contains("clamp_nib_visual_rect("), "{cc}");
    }
}
