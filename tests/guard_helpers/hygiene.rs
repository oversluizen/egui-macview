//! Entrypoint safety, dead-code, language/path hygiene, and JSON schema helpers for architecture guards.
#![allow(dead_code)]

use super::strip_comments_and_strings;
use std::collections::BTreeSet;

/// Verifies Windows GUI Subsystem & Safe `eframe` Startup invariants on GUI entrypoints.
pub fn check_gui_entrypoint_safety(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    if !stripped.contains("fn main") {
        return Vec::new();
    }
    let mut violations = Vec::new();
    if !source.contains("#![cfg_attr(not(test), windows_subsystem = \"windows\")]") {
        violations.push(format!("{file_label} -> missing `#![cfg_attr(not(test), windows_subsystem = \"windows\")]`"));
    }
    if stripped.contains("eframe::run_native") || stripped.contains("NativeOptions") {
        if stripped.contains("NativeOptions::default()") {
            violations.push(format!("{file_label} -> bare `NativeOptions::default()` is forbidden (Safe eframe Startup Guard)"));
        }
        for required in ["Renderer::Glow", "InstanceFlags::empty()", "system_font_fallback: false", "persist_window: false"] {
            if !stripped.contains(required) {
                violations.push(format!("{file_label} -> `eframe::run_native` missing safe startup setting `{required}`"));
            }
        }
    }
    violations
}

/// Forbids `allow(dead_code)` in `src/` production modules (Zero Dead Code Guard).
pub fn check_zero_dead_code_in_source(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    let mut violations = Vec::new();
    for (idx, line) in stripped.lines().enumerate() {
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.contains("allow(dead_code)") {
            violations.push(format!("{file_label}:{} -> forbidden `allow(dead_code)` in production `src/`", idx + 1));
        }
    }
    violations
}

/// Checks text for internal vault references, local paths, pseudo-academic jargon, and Dutch words.
pub fn check_vault_leakage_and_dutch_in_text(file_label: &str, content: &str) -> Vec<String> {
    let forbidden_terms: Vec<String> = vec![
        format!("{}-{}", "GL", "1015"),
        format!("{}-{}", "GL", "1005"),
        format!("{}-{}", "GL", "1016"),
        format!("{}-{}", "GL", "1017"),
        format!("{}-{}", "SOP", "1016"),
        format!("{} {}{}", "De", "Verankerings", "wet"),
        format!("{} {}", "06", "AI Team"),
        format!("{}:/{}", "D", "Programmeren"),
        format!("{}:\\{}", "D", "Programmeren"),
        format!("{}:/{}", "G", "My Drive"),
        format!("{}-{}", "Jacos", "Air"),
        format!("{} {}", "Damped Harmonic Oscillator", "Tensor Manifold"),
        format!("{} {}", "Mathematical Layout Determinism", "with Subpixel Snap Rasterization"),
        format!("{} {}", "Polymorphic transactional", "VC state machines"),
    ];
    let mut violations = Vec::new();
    for term in &forbidden_terms {
        if content.contains(term.as_str()) {
            violations.push(format!("{file_label} contains forbidden term `{term}`"));
        }
    }
    let dutch_vocab_encoded = "vo|or,na|ar,he|t,e|en,zi|jn,wor|dt,wor|den,ni|et,ge|en,ma|ar,\
        oo|k,al|le,de|ze,ech|ter,ind|ien,wan|neer,zo|als,om|dat,zo|dat,waa|rin,waa|rbij,tus|sen,\
        bin|nen,bui|ten,zon|der,te|gen,van|af,vol|gens,tij|dens,ech|te,vol|ledig,ver|plicht,\
        ver|boden,geb|ruikt,geb|ruiken,al|leen,alt|ijd,noo|it,mo|et,moe|ten,wel|ke,ied|ere,el|ke,\
        enk|ele,bei|de,and|ere,nie|uwe,eer|ste,laat|ste,vol|gende,hui|dige,jui|ste,waa|rom,\
        sch|erm,ven|ster,kn|op,kno|ppen,kl|eur,kleu|ren,bree|dte,hoo|gte,ach|tergrond,vo|orgrond,\
        tek|st,gro|otte,afs|tand,stan|daard,bes|tand,best|anden,re|gel,reg|els,fo|ut,fou|ten,\
        bere|kening,weer|gave,spook|code,verank|eringswet,stam|boom,vibe|coding";
    let dutch_words: BTreeSet<String> = dutch_vocab_encoded.split(',').map(|s| s.replace('|', "")).collect();
    for (idx, line) in content.lines().enumerate() {
        for word in line.split(|c: char| !c.is_ascii_alphabetic()) {
            if word.len() >= 3 {
                let lower = word.to_ascii_lowercase();
                if dutch_words.contains(&lower) {
                    violations.push(format!("{file_label}:{} contains non-English (Dutch) word `{word}`", idx + 1));
                }
            }
        }
    }
    violations
}

/// Parses `[dependencies]` crate keys from `Cargo.toml`.
pub fn extract_cargo_dependencies(cargo_toml: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for raw_line in cargo_toml.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_deps = line == "[dependencies]";
            continue;
        }
        if in_deps {
            if let Some((pkg, _)) = line.split_once('=') {
                let crate_ident = pkg.trim().replace('-', "_");
                if !crate_ident.is_empty() {
                    deps.push(crate_ident);
                }
            }
        }
    }
    deps
}

pub fn collect_json_leaf_paths(val: &serde_json::Value, prefix: String, is_in_metrics_or_color: bool, out: &mut BTreeSet<String>) {
    if let Some(obj) = val.as_object() {
        let is_dtcg_leaf = obj.contains_key("$value")
            || (obj.contains_key("value") && obj.contains_key("$type"))
            || (obj.contains_key("mass") && obj.contains_key("stiffness"))
            || (obj.contains_key("size_pt") && obj.contains_key("tracking_pt"));
        if is_dtcg_leaf && is_in_metrics_or_color {
            out.insert(prefix);
            return;
        }
        for (k, v) in obj {
            if k.starts_with('$') || k == "polynomial_spec001" {
                continue;
            }
            let next = if prefix.is_empty() { k.clone() } else { format!("{prefix}.{k}") };
            let track = is_in_metrics_or_color
                || next.starts_with("macview.metrics")
                || next.starts_with("macview.color")
                || next.starts_with("macview.motion")
                || next.starts_with("macview.typography.tracking_table");
            collect_json_leaf_paths(v, next, track, out);
        }
    }
}

/// Checks scene purity, declarative bindings, and test-suite architecture hygiene (`REQ-001-06`, `REQ-013-07`).
pub fn check_scene_and_test_purity(manifest_dir: &std::path::Path) -> Vec<String> {
    use std::fs;
    let mut violations = Vec::new();
    for no_json_rel in [
        "src/scene/nib_frontend.rs",
        "src/scene/nib_ibtool.rs",
        "src/scene/nib_ibtool_widgets.rs",
        "src/scene/nib_varint.rs",
        "src/scene/xib_exporter.rs",
        "src/scene/xib_strings.rs",
    ] {
        let src = fs::read_to_string(manifest_dir.join(no_json_rel)).expect("read scene file");
        if src.contains("serde_json") || src.contains("macviewMeta") {
            violations.push(format!("{no_json_rel} must not use serde_json or macviewMeta JSON shortcuts (REQ-013-05.5)"));
        }
        for bad_ib in ["\"ibTip\"", "\"ibTitle\"", "\"ibSize\"", "\"ibIcon\"", "\"ibDesignFrame\"", "\"ibId\""] {
            if src.contains(bad_ib) {
                violations.push(format!("{no_json_rel} contains forbidden custom runtime attribute {bad_ib} (REQ-013-07.1)"));
            }
        }
    }
    for scene_rel in [
        "src/scene/mod.rs",
        "src/scene/ir.rs",
        "src/scene/nib_ibtool.rs",
        "src/scene/nib_ibtool_widgets.rs",
        "src/scene/runtime.rs",
        "src/scene/runtime_widgets.rs",
        "src/scene/storyboard.rs",
    ] {
        let src = fs::read_to_string(manifest_dir.join(scene_rel)).expect("read scene pipeline file");
        for bad_heuristic in [
            "== \"1\"",
            "!= \"1\"",
            "10_000 +",
            "\"statusText\"",
            "\"GSFontViewWidth\"",
            "starts_with(\"remove\")",
            "starts_with(\"delete\")",
            "starts_with(\"add\")",
            "is_bot_parent",
            "frame[1] + frame[3] <= 40.0",
            "design_size[1] > 420.0",
        ] {
            if src.contains(bad_heuristic) {
                violations.push(format!(
                    "{scene_rel} contains forbidden non-SSOT heuristic `{bad_heuristic}` (REQ-013-10.2 Pure Pipeline Guard)"
                ));
            }
        }
    }
    if let Ok(entries) = fs::read_dir(manifest_dir.join("assets/nib_showcase")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("xib") {
                let xib_txt = fs::read_to_string(&p).expect("read xib");
                for bad_ib in [
                    "keyPath=\"ibTip\"",
                    "keyPath=\"ibTitle\"",
                    "keyPath=\"ibSize\"",
                    "keyPath=\"ibIcon\"",
                    "keyPath=\"ibDesignFrame\"",
                    "keyPath=\"ibId\"",
                ] {
                    if xib_txt.contains(bad_ib) {
                        violations.push(format!("{:?} contains forbidden {bad_ib} (REQ-013-07.1)", p.file_name()));
                    }
                }
            }
        }
    }
    let rt_tabs = fs::read_to_string(manifest_dir.join("src/scene/runtime_tabs.rs")).expect("read runtime_tabs.rs");
    for bad_tab in [
        "AVPlayer HUD",
        "Collection Grid",
        "Clock & Calendar",
        "MacPlayerView",
        "MacCollectionView",
        "MacDatePicker",
        "macview_tab_collection_count",
    ] {
        if rt_tabs.contains(bad_tab) {
            violations.push(format!("src/scene/runtime_tabs.rs contains hardcoded demo tab logic `{bad_tab}` (REQ-013-07.2)"));
        }
    }
    let rt_widgets = fs::read_to_string(manifest_dir.join("src/scene/runtime_widgets.rs")).expect("read runtime_widgets.rs");
    for bad_w in ["\"ProRes\"", "\"HEVC\"", "\"Created Date\"", "macview_tab_collection_count", "-10_000.0..=10_000.0"] {
        if rt_widgets.contains(bad_w) {
            violations.push(format!("src/scene/runtime_widgets.rs contains hardcoded widget data `{bad_w}` (REQ-013-07.2)"));
        }
    }
    for (gal_rel, bad_list) in [
        ("examples/gallery/state1_canvas.rs", &["\"Normal\"", "\"Multiply\""][..]),
        ("examples/gallery/state2_symbols_layout.rs", &["\"Inter Regular (OFL)\"", "\"Semibold\""][..]),
        ("examples/gallery/state3_command_state.rs", &["\"Apple ProRes\"", "\"H.264\""][..]),
        ("examples/gallery/nib_actions.rs", &["\"ProRes 4444 XQ\"", "\"WAV 96kHz\""][..]),
        ("examples/gallery/nib_showcase_builders.rs", &["starts_with(\"s6-en-"][..]),
    ] {
        let gsrc = fs::read_to_string(manifest_dir.join(gal_rel)).expect("read gallery file");
        for bad_str in bad_list {
            if gsrc.contains(bad_str) {
                violations.push(format!("{gal_rel} contains hardcoded .xib duplicate `{bad_str}` (REQ-013-07.3)"));
            }
        }
    }
    let thumb_ov = fs::read_to_string(manifest_dir.join("examples/gallery/state1_thumb_overlay.rs")).unwrap_or_default();
    if thumb_ov.contains("sp.rect(p_rect") || thumb_ov.contains("sp.rect(s_rect") {
        violations.push("examples/gallery/state1_thumb_overlay.rs contains hand-painted `sp.rect` fake mode strips instead of MacSegmentedControl/MacColorWell".into());
    }
    let builders = fs::read_to_string(manifest_dir.join("examples/gallery/nib_showcase_builders.rs")).unwrap_or_default();
    if builders.contains("(65.0..=155.0)") || builders.contains("(170.0..=255.0)") {
        violations.push("examples/gallery/nib_showcase_builders.rs contains hardcoded Y-coordinate ranges `(65.0..=155.0)` / `(170.0..=255.0)` instead of declarative `enabled_binding`".into());
    }
    let bad_ctors = [
        "MacButtonCell::new",
        "MacSegmentedControl::new",
        "MacPopUpButton::new",
        "MacCheckbox::new",
        "MacColorWell::new",
        "MacStepper::new",
        "MacComboBox::new",
    ];
    for rel_file in [
        "examples/gallery/state1_canvas.rs",
        "examples/gallery/state2_symbols_layout.rs",
        "examples/gallery/state3_command_state.rs",
        "examples/gallery/state4_finder_browser.rs",
        "examples/gallery/state5_avkit_media.rs",
        "examples/gallery/inspector.rs",
        "examples/gallery/inspector_nib.rs",
    ] {
        let src = fs::read_to_string(manifest_dir.join(rel_file)).expect("read screen source");
        for bad_ctor in bad_ctors {
            if src.contains(bad_ctor) {
                violations.push(format!("REQ-013-02P: {rel_file} must not construct `{bad_ctor}` imperatively"));
            }
        }
    }
    for test_rel in [
        "tests/gallery_binding_tests.rs",
        "tests/runtime_hygiene_and_invariants_tests.rs",
        "tests/media_and_finder_interactive_boost_tests.rs",
        "tests/palette_and_footer_tests.rs",
        "tests/spec015_dual_visual_and_pro_inspector_tests.rs",
        "tests/symbol_tests.rs",
    ] {
        let tsrc = fs::read_to_string(manifest_dir.join(test_rel)).unwrap_or_default();
        if tsrc.contains("read_all_gallery_sources") {
            violations.push(format!("{test_rel} contains forbidden `read_all_gallery_sources` string-grep anti-pattern"));
        }
        if tsrc.contains("studio_controls_v05.rs") {
            violations.push(format!("{test_rel} contains forbidden legacy `studio_controls_v05.rs` monolith import"));
        }
    }
    violations
}
