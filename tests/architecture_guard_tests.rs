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

const UI_COMPONENT_DIRS: &[&str] = &[
    "src/controls",
    "src/layout",
    "src/surfaces",
    "src/navigation",
    "src/feedback",
];
const EXTRA_GUARDED_FILES: &[&str] = &["src/theme/mod.rs", "src/theme/accessors.rs"];

#[test]
fn test_guard_zero_magic_numbers_and_colors_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut magic_v, mut color_v) = (Vec::new(), Vec::new());
    for dir_rel in UI_COMPONENT_DIRS {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("read source");
            magic_v.extend(check_magic_numbers_in_source(&rel, &content));
            color_v.extend(check_hardcoded_colors_in_source(&rel, &content));
        }
    }
    for file_rel in EXTRA_GUARDED_FILES {
        let content = fs::read_to_string(manifest_dir.join(file_rel)).expect("read source");
        magic_v.extend(check_magic_numbers_in_source(file_rel, &content));
        color_v.extend(check_hardcoded_colors_in_source(file_rel, &content));
    }
    assert!(magic_v.is_empty(), "Magic Number Guard:\n{magic_v:#?}");
    assert!(color_v.is_empty(), "Hardcoded Color Guard:\n{color_v:#?}");
}

#[test]
fn test_guard_zero_token_arithmetic_and_domain_purity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (mut math_v, mut domain_v, mut salt_v) = (Vec::new(), Vec::new(), Vec::new());
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file
            .strip_prefix(manifest_dir)
            .unwrap_or(&file)
            .to_string_lossy();
        let content = fs::read_to_string(&file).expect("read source");
        math_v.extend(check_token_arithmetic_cheating(&rel, &content));
        domain_v.extend(check_domain_leakage_in_source(&rel, &content));
        salt_v.extend(check_uibuilder_id_salt_in_source(&rel, &content));
    }
    for file in collect_rs_files(&manifest_dir.join("examples")) {
        let rel = file
            .strip_prefix(manifest_dir)
            .unwrap_or(&file)
            .to_string_lossy();
        let content = fs::read_to_string(&file).expect("read source");
        domain_v.extend(check_domain_leakage_in_source(&rel, &content));
        salt_v.extend(check_uibuilder_id_salt_in_source(&rel, &content));
    }
    assert!(math_v.is_empty(), "Token Math Guard:\n{math_v:#?}");
    assert!(domain_v.is_empty(), "Domain Purity Guard:\n{domain_v:#?}");
    assert!(
        salt_v.is_empty(),
        "UiBuilder id_salt Guard:\n{salt_v:#?}"
    );
}

#[test]
fn test_guard_token_catalog_100_percent_schema_synchronization() {
    let raw_json = include_str!("../tokens/design_tokens.json");
    let raw_val: serde_json::Value = serde_json::from_str(raw_json).expect("Valid JSON");
    let typed_tokens = egui_macview::tokens::MacDesignTokens::load_default();
    let reserialized_val = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || serde_json::to_value(&typed_tokens).expect("Serialize MacDesignTokens"))
        .expect("spawn")
        .join()
        .expect("join");
    let (mut raw_leaves, mut typed_leaves) = (BTreeSet::new(), BTreeSet::new());
    collect_json_leaf_paths(&raw_val, String::new(), false, &mut raw_leaves);
    collect_json_leaf_paths(&reserialized_val, String::new(), false, &mut typed_leaves);
    let missing: Vec<_> = raw_leaves.difference(&typed_leaves).collect();
    assert!(missing.is_empty(), "Orphaned JSON tokens:\n{missing:#?}");
}

#[test]
fn test_guard_documentation_and_file_size_ceilings() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).expect("read lib.rs");
    assert!(lib_rs.contains("#![deny(missing_docs)]"));
    let readme = fs::read_to_string(manifest_dir.join("README.md")).expect("read README.md");
    assert!(
        !readme.contains("](src/"),
        "README.md is embedded in src/lib.rs via include_str! and must use rustdoc intra-doc links [`Type`] instead of relative (src/...) file links that 404 on docs.rs"
    );
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let content = fs::read_to_string(&file).expect("read source");
        assert!(content
            .lines()
            .take(5)
            .any(|l| l.trim_start().starts_with("//!")));
    }
    let mut violations = Vec::new();
    for dir_rel in &["src", "tests", "examples"] {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("read source");
            let (physical, esloc) = count_lines_and_esloc(&content);
            if physical >= MAX_PHYSICAL_LINES || esloc >= MAX_ESLOC {
                violations.push(format!(
                    "{rel} exceeds ceiling: physical={physical} (<{MAX_PHYSICAL_LINES}), eSLOC={esloc} (<{MAX_ESLOC})"
                ));
            }
            for (line_idx, line) in content.lines().enumerate() {
                let code_part = line.split("//").next().unwrap_or("").trim();
                if !code_part.contains('"') && !code_part.ends_with('\\') {
                    let without_arrays = code_part
                        .replace("; 2]", "]")
                        .replace("; 3]", "]")
                        .replace("; 4]", "]");
                    if without_arrays.matches(';').count() >= 3 {
                        violations.push(format!(
                            "{rel}:{} -> single-line statement cramming (>=3 semicolons)",
                            line_idx + 1
                        ));
                    }
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
        "box_view",
        "button",
        "collection_view",
        "color_well",
        "combo_box",
        "date_picker",
        "disclosure_group",
        "level_indicator",
        "outline_view",
        "path_control",
        "player_view",
        "popover_metrics",
        "popup_button",
        "progress_indicator",
        "rule_editor",
        "scroll_view",
        "search_field",
        "segmented",
        "slider",
        "split_view",
        "stack_view",
        "stepper",
        "tab_view",
        "table_view",
        "thumbnail",
        "toggle",
        "token_field",
        "trim_scrubber",
    ];
    for file_path in collect_rs_files(&manifest_dir.join("src/controls")) {
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let pname = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if fname == "mod.rs" && pname == "controls" {
            continue;
        }
        let allowed: &[&str] = match (pname, fname) {
            (_, "box_view.rs") => &["box_view"],
            (_, "button_cell.rs") => &["button"],
            (_, "circular_slider.rs") => &["slider"],
            (_, "collection_view.rs") => &["collection_view"],
            (_, "color_well.rs") => &["color_well"],
            (_, "combo_box.rs") => &["combo_box"],
            (_, "date_picker.rs") | ("date_picker", _) => &["date_picker"],
            (_, "disclosure_group.rs") => &["disclosure_group"],
            (_, "level_indicator.rs") => &["level_indicator"],
            (_, "outline_view.rs") => &["outline_view", "thumbnail"],
            (_, "path_control.rs") => &["path_control"],
            (_, "player_view.rs") | ("player_view", _) => &["player_view"],
            (_, "popup_button.rs") => &["popup_button"],
            (_, "progress_indicator.rs") => &["progress_indicator"],
            (_, "rating_indicator.rs") => &["level_indicator"],
            (_, "rule_editor.rs") => &["rule_editor"],
            (_, "search_field.rs") => &["search_field"],
            (_, "segmented.rs") => &["segmented"],
            (_, "slider.rs") => &["slider"],
            (_, "stepper.rs") => &["stepper"],
            (_, "tab_view.rs") => &["tab_view"],
            ("table_view", _) => &["table_view"],
            (_, "thumbnail_cell.rs") => &["thumbnail"],
            (_, "toggle.rs") => &["toggle"],
            (_, "token_field.rs") => &["token_field"],
            ("trim_scrubber", _) => &["trim_scrubber"],
            _ => &[],
        };
        let stripped = strip_comments_and_strings(&fs::read_to_string(&file_path).unwrap());
        for &m in &methods {
            if !allowed.contains(&m) {
                assert!(
                    !stripped.contains(&format!("theme.{m}(")),
                    "{fname} calls theme.{m}("
                );
            }
        }
    }
    assert!(!check_magic_numbers_in_source("bad.rs", "let h = 18.0;").is_empty());
    assert!(check_magic_numbers_in_source("ok.rs", "let mid = w * 0.5;").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = Color32::WHITE;").is_empty());
    assert!(
        !check_token_arithmetic_cheating("bad.rs", "let w = theme.width.points() + 4.0;")
            .is_empty()
    );
    assert!(!check_domain_leakage_in_source("bad.rs", "struct PedigreeNode;").is_empty());
    assert!(!check_uibuilder_id_salt_in_source(
        "bad.rs",
        "ui.new_child(UiBuilder::new().max_rect(r));"
    )
    .is_empty());
    assert!(check_uibuilder_id_salt_in_source(
        "ok.rs",
        "ui.new_child(UiBuilder::new().id_salt(\"s\").max_rect(r));"
    )
    .is_empty());
}

#[test]
fn test_guard_windows_gui_subsystem_and_safe_eframe_startup() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();
    let mut checked_entrypoints = 0usize;
    for dir_rel in &["examples", "src/bin"] {
        let dir_path = manifest_dir.join(dir_rel);
        if !dir_path.exists() {
            continue;
        }
        for file in collect_rs_files(&dir_path) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("read entrypoint candidate");
            if strip_comments_and_strings(&content).contains("fn main") {
                checked_entrypoints += 1;
            }
            violations.extend(check_gui_entrypoint_safety(&rel, &content));
        }
    }
    assert!(
        checked_entrypoints >= 3,
        "Expected at least 3 GUI entrypoints in examples/"
    );
    assert!(
        violations.is_empty(),
        "Windows GUI Subsystem & Safe eframe Startup Guard:\n{violations:#?}"
    );

    // Self-check: bare NativeOptions::default() without windows_subsystem or safe flags must fail
    let bad_gui = "fn main() { let _ = eframe::run_native(\"app\", eframe::NativeOptions::default(), Box::new(|_| unreachable!())); }";
    let self_check = check_gui_entrypoint_safety("bad_example.rs", bad_gui);
    assert!(
        self_check.len() >= 5,
        "Self-check must catch missing subsystem + 4 eframe safety flags: {self_check:#?}"
    );
}

#[test]
fn test_guard_zero_dead_code_and_zero_unused_dependencies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut dead_code_violations = Vec::new();
    let mut combined_src = String::new();

    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file
            .strip_prefix(manifest_dir)
            .unwrap_or(&file)
            .to_string_lossy();
        let content = fs::read_to_string(&file).expect("read src file");
        dead_code_violations.extend(check_zero_dead_code_in_source(&rel, &content));
        combined_src.push_str(&strip_comments_and_strings(&content));
        combined_src.push('\n');
    }
    assert!(
        dead_code_violations.is_empty(),
        "Zero Dead Code Guard (`allow(dead_code)` forbidden in `src/`):\n{dead_code_violations:#?}"
    );
    assert!(
        !check_zero_dead_code_in_source("src/bad.rs", "#[allow(dead_code)]\nstruct Ghost;")
            .is_empty()
    );

    let cargo_toml = fs::read_to_string(manifest_dir.join("Cargo.toml")).expect("read Cargo.toml");
    let deps = extract_cargo_dependencies(&cargo_toml);
    assert!(
        !deps.is_empty(),
        "Cargo.toml [dependencies] must not be empty"
    );

    let mut unused_deps = Vec::new();
    for dep in &deps {
        let used = combined_src.contains(&format!("{dep}::"))
            || combined_src.contains(&format!("use {dep}"))
            || (dep == "serde"
                && (combined_src.contains("Serialize") || combined_src.contains("Deserialize")));
        if !used {
            unused_deps.push(dep.clone());
        }
    }
    assert!(
        unused_deps.is_empty(),
        "Zero Unused Dependencies Guard: crates in [dependencies] unused in `src/`:\n{unused_deps:#?}"
    );

    // State-Invariant & Undo/Redo Roundtrip + Cold-Start Smoke Guard
    let undo_src =
        fs::read_to_string(manifest_dir.join("src/state/undo.rs")).expect("read undo.rs");
    assert!(
        undo_src.contains("pub fn validate_invariants("),
        "NSUndoManager in `src/state/undo.rs` must expose `pub fn validate_invariants(&self)`"
    );
    let hygiene_test =
        fs::read_to_string(manifest_dir.join("tests/runtime_hygiene_and_invariants_tests.rs"))
            .expect("read runtime_hygiene_and_invariants_tests.rs");
    assert!(
        hygiene_test.contains("validate_invariants()")
            && hygiene_test.contains(".undo(")
            && hygiene_test.contains(".redo(")
            && hygiene_test.contains("cold_start")
            && hygiene_test.contains("repaint_delay"),
        "Integration suite must verify Cold-Start (<500ms), Zero Idle Repaint Leak, and Action->Undo->Redo invariants"
    );
}

#[test]
fn test_guard_zero_vault_leakage_and_de_ai_ification() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();

    for dir in &["src", "tests", "specs", "examples"] {
        for file in collect_rs_files(&manifest_dir.join(dir)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("read file");
            violations.extend(check_vault_leakage_and_dutch_in_text(&rel, &content));
        }
    }

    for dir_path in [
        manifest_dir.to_path_buf(),
        manifest_dir.join("specs"),
        manifest_dir.join("tokens"),
    ] {
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && matches!(
                        path.extension().and_then(|s| s.to_str()),
                        Some("md" | "toml" | "json" | "cmd")
                    )
                {
                    let rel = path
                        .strip_prefix(manifest_dir)
                        .unwrap_or(&path)
                        .to_string_lossy();
                    let content = fs::read_to_string(&path).expect("read file");
                    violations.extend(check_vault_leakage_and_dutch_in_text(&rel, &content));
                }
            }
        }
    }

    assert!(
        !check_vault_leakage_and_dutch_in_text(
            "dutch_sample.rs",
            &format!("// {}{} {}{}", "vo", "or", "kn", "op")
        )
        .is_empty(),
        "Self-check must catch non-English (Dutch) comments"
    );
    assert!(
        violations.is_empty(),
        "Vault Leakage, Local Path, Jargon & Non-English (Dutch) Guard:\n{violations:#?}"
    );
}

