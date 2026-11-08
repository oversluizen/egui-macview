//! Automated Architecture Guard Suite (Fitness Functions).
//!
//! Normative reference: GL-1015 (Rules 8, 9, 10, 11, 13: Zero Hardcoding, Zero Token Cheating,
//! Domain-Agnostic Separation, Documentation Completeness, and File Size Ceilings).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED_RATIOS: &[f64] = &[0.0, 0.5, 1.0, 2.0];
const FORBIDDEN_DOMAIN_TERMS: &[&str] = &[
    "pedigree", "gedcom", "kinship", "ancestor", "proband", "marriage", "ahnen", "stamboom",
    "divorce", "adoptive", "foster", "sibling",
];
const MAX_PHYSICAL_LINES: usize = 599;
const MAX_ESLOC: usize = 400;

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Strips comments (`//`, `/* ... */`) and string literals (`"..."`) while preserving line count.
pub fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let n = chars.len();

    while i < n {
        if i + 1 < n && chars[i] == '/' && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < n && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1usize;
            while i < n && depth > 0 {
                if i + 1 < n && chars[i] == '/' && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if i + 1 < n && chars[i] == '*' && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            continue;
        }
        if chars[i] == 'r' && i + 1 < n && (chars[i + 1] == '#' || chars[i + 1] == '"') {
            let mut hashes = 0usize;
            let mut j = i + 1;
            while j < n && chars[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if j < n && chars[j] == '"' {
                out.push('"');
                i = j + 1;
                while i < n {
                    if chars[i] == '"' {
                        let mut k = 0;
                        while i + 1 + k < n && chars[i + 1 + k] == '#' && k < hashes {
                            k += 1;
                        }
                        if k == hashes {
                            i += 1 + hashes;
                            break;
                        }
                    }
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
                out.push('"');
                continue;
            }
        }
        if chars[i] == '"' {
            out.push('"');
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                if chars[i] == '\n' {
                    out.push('\n');
                }
                i += 1;
            }
            out.push('"');
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Counts physical lines and effective SLOC (non-empty, non-comment lines).
pub fn count_lines_and_esloc(source: &str) -> (usize, usize) {
    let physical = source.lines().count();
    let stripped = strip_comments_and_strings(source);
    let esloc = stripped.lines().filter(|l| !l.trim().is_empty()).count();
    (physical, esloc)
}

pub fn extract_float_literals(code: &str) -> Vec<(usize, String, f64)> {
    let mut results = Vec::new();
    for (line_idx, line) in code.lines().enumerate() {
        let line_num = line_idx + 1;
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            let b = bytes[i];
            if b.is_ascii_alphabetic() || b == b'_' {
                while i < len
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'.')
                {
                    i += 1;
                }
                continue;
            }
            if b.is_ascii_digit() {
                let start = i;
                while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                    i += 1;
                }
                if i + 1 < len
                    && bytes[i] == b'.'
                    && bytes[i + 1].is_ascii_digit()
                    && (i + 1 >= len || bytes[i + 1] != b'.')
                {
                    i += 1;
                    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                        i += 1;
                    }
                    let num_end = i;
                    if i + 3 <= len && (&line[i..i + 3] == "f32" || &line[i..i + 3] == "f64") {
                        i += 3;
                    }
                    let raw = &line[start..i];
                    let numeric_part = line[start..num_end].replace('_', "");
                    if let Ok(val) = numeric_part.parse::<f64>() {
                        results.push((line_num, raw.to_string(), val));
                    }
                    continue;
                }
                if i + 3 <= len && (&line[i..i + 3] == "f32" || &line[i..i + 3] == "f64") {
                    let num_end = i;
                    i += 3;
                    let raw = &line[start..i];
                    let numeric_part = line[start..num_end].replace('_', "");
                    if let Ok(val) = numeric_part.parse::<f64>() {
                        results.push((line_num, raw.to_string(), val));
                    }
                    continue;
                }
                continue;
            }
            i += 1;
        }
    }
    results
}

pub fn check_magic_numbers_in_source(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    let mut violations = Vec::new();
    for (line_num, raw, val) in extract_float_literals(&stripped) {
        let is_allowed = ALLOWED_RATIOS
            .iter()
            .any(|&allowed| (val - allowed).abs() < 1e-9);
        if !is_allowed {
            violations.push(format!(
                "{}:{} -> forbidden magic float literal `{}` (value = {}). Must be sourced from design tokens!",
                file_label, line_num, raw, val
            ));
        }
    }
    violations
}

pub fn check_hardcoded_colors_in_source(file_label: &str, source: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let stripped = strip_comments_and_strings(source);

    let forbidden_calls = [
        "Color32::from_rgb(",
        "Color32::from_rgba_premultiplied(",
        "Color32::from_rgba_unmultiplied(",
        "Color32::from_black_alpha(",
        "Color32::WHITE",
    ];

    for (idx, line) in stripped.lines().enumerate() {
        let line_num = idx + 1;
        for call in &forbidden_calls {
            if line.contains(call) {
                violations.push(format!(
                    "{}:{} -> forbidden hardcoded color `{}` in UI layer.",
                    file_label, line_num, call
                ));
            }
        }
    }

    let mut in_doc = false;
    for (idx, raw_line) in source.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        let _ = &mut in_doc;
        if let Some(pos) = raw_line.find("\"#") {
            let after = &raw_line[pos + 2..];
            let hex_len = after.chars().take_while(|c| c.is_ascii_hexdigit()).count();
            if (hex_len == 6 || hex_len == 8) && after.as_bytes().get(hex_len) == Some(&b'"') {
                violations.push(format!(
                    "{}:{} -> forbidden inline hex color string `\"#{}\"` in UI code.",
                    file_label,
                    idx + 1,
                    &after[..hex_len]
                ));
            }
        }
    }
    violations
}

pub fn check_token_arithmetic_cheating(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    let mut violations = Vec::new();
    for (idx, line) in stripped.lines().enumerate() {
        let line_num = idx + 1;
        if let Some(pos) = line.find(".points()") {
            let after = line[pos + ".points()".len()..].trim_start();
            if after.starts_with('+') || after.starts_with('-') {
                let expr_rest = after[1..].trim_start();
                let floats = extract_float_literals(expr_rest);
                for (_, raw, val) in floats {
                    let is_allowed = ALLOWED_RATIOS
                        .iter()
                        .any(|&allowed| (val - allowed).abs() < 1e-9);
                    if !is_allowed && expr_rest.starts_with(&raw) {
                        violations.push(format!(
                            "{}:{} -> token arithmetic cheating detected: `.points() {} {}`.",
                            file_label,
                            line_num,
                            &after[..1],
                            raw
                        ));
                    }
                }
            }
        }
    }
    violations
}

pub fn check_domain_leakage_in_source(file_label: &str, source: &str) -> Vec<String> {
    let mut violations = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();
        for &term in FORBIDDEN_DOMAIN_TERMS {
            if lower.contains(term) {
                violations.push(format!(
                    "{}:{} -> domain leakage detected: forbidden term `{}` found in `{}`",
                    file_label,
                    idx + 1,
                    term,
                    line.trim()
                ));
            }
        }
    }
    violations
}

fn collect_json_leaf_paths(
    val: &serde_json::Value,
    prefix: String,
    is_in_metrics_or_color: bool,
    out: &mut BTreeSet<String>,
) {
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
            let next_prefix = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{}.{}", prefix, k)
            };
            let track = is_in_metrics_or_color
                || next_prefix.starts_with("macview.metrics")
                || next_prefix.starts_with("macview.color")
                || next_prefix.starts_with("macview.motion")
                || next_prefix.starts_with("macview.typography.tracking_table");
            collect_json_leaf_paths(v, next_prefix, track, out);
        }
    }
}

const UI_COMPONENT_DIRS: &[&str] = &[
    "src/controls",
    "src/layout",
    "src/surfaces",
    "src/navigation",
    "src/feedback",
];
const EXTRA_GUARDED_FILES: &[&str] = &["src/theme/mod.rs", "src/theme/accessors.rs"];

#[test]
fn test_guard_zero_magic_numbers_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for dir_rel in UI_COMPONENT_DIRS {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("Failed to read source file");
            all_violations.extend(check_magic_numbers_in_source(&rel, &content));
        }
    }
    for file_rel in EXTRA_GUARDED_FILES {
        let file = manifest_dir.join(file_rel);
        let content = fs::read_to_string(&file).expect("Failed to read source file");
        all_violations.extend(check_magic_numbers_in_source(file_rel, &content));
    }
    assert!(
        all_violations.is_empty(),
        "GL-1015 Magic Number Guard Failed:\n{}",
        all_violations.join("\n")
    );
}

#[test]
fn test_guard_zero_hardcoded_colors_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for dir_rel in UI_COMPONENT_DIRS {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("Failed to read source file");
            all_violations.extend(check_hardcoded_colors_in_source(&rel, &content));
        }
    }
    for file_rel in EXTRA_GUARDED_FILES {
        let file = manifest_dir.join(file_rel);
        let content = fs::read_to_string(&file).expect("Failed to read source file");
        all_violations.extend(check_hardcoded_colors_in_source(file_rel, &content));
    }
    assert!(
        all_violations.is_empty(),
        "GL-1015 Hardcoded Color Guard Failed:\n{}",
        all_violations.join("\n")
    );
}

#[test]
fn test_guard_zero_token_arithmetic_cheating() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file
            .strip_prefix(manifest_dir)
            .unwrap_or(&file)
            .to_string_lossy();
        let content = fs::read_to_string(&file).expect("Failed to read source file");
        all_violations.extend(check_token_arithmetic_cheating(&rel, &content));
    }
    assert!(
        all_violations.is_empty(),
        "GL-1015 Token Arithmetic Cheating Guard Failed:\n{}",
        all_violations.join("\n")
    );
}

#[test]
fn test_guard_domain_agnostic_purity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut all_violations = Vec::new();
    for dir_rel in &["src", "examples"] {
        let dir = manifest_dir.join(dir_rel);
        if dir.exists() {
            for file in collect_rs_files(&dir) {
                let rel = file
                    .strip_prefix(manifest_dir)
                    .unwrap_or(&file)
                    .to_string_lossy();
                let content = fs::read_to_string(&file).expect("Failed to read source file");
                all_violations.extend(check_domain_leakage_in_source(&rel, &content));
            }
        }
    }
    assert!(
        all_violations.is_empty(),
        "GL-1015 Domain-Agnostic Purity Guard Failed:\n{}",
        all_violations.join("\n")
    );
}

#[test]
fn test_guard_token_catalog_100_percent_schema_synchronization() {
    let raw_json = include_str!("../tokens/design_tokens.json");
    let raw_val: serde_json::Value =
        serde_json::from_str(raw_json).expect("Valid JSON in design_tokens.json");
    let typed_tokens = egui_macview::tokens::MacDesignTokens::from_json(raw_json)
        .expect("Typed MacDesignTokens must deserialize design_tokens.json");
    let reserialized_val =
        serde_json::to_value(&typed_tokens).expect("MacDesignTokens must serialize back to Value");

    let mut raw_leaves = BTreeSet::new();
    collect_json_leaf_paths(&raw_val, String::new(), false, &mut raw_leaves);
    let mut typed_leaves = BTreeSet::new();
    collect_json_leaf_paths(&reserialized_val, String::new(), false, &mut typed_leaves);

    let missing_in_rust: Vec<_> = raw_leaves.difference(&typed_leaves).collect();
    assert!(
        missing_in_rust.is_empty(),
        "Orphaned JSON tokens in design_tokens.json not mapped in Rust structs:\n{:#?}",
        missing_in_rust
    );
}

#[test]
fn test_guard_documentation_completeness() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_rs = fs::read_to_string(manifest_dir.join("src/lib.rs")).expect("Failed to read lib.rs");
    assert!(
        lib_rs.contains("#![deny(missing_docs)]"),
        "src/lib.rs must enforce `#![deny(missing_docs)]`"
    );

    for file in collect_rs_files(&manifest_dir.join("src")) {
        let rel = file
            .strip_prefix(manifest_dir)
            .unwrap_or(&file)
            .to_string_lossy();
        let content = fs::read_to_string(&file).expect("Failed to read source file");
        let has_module_doc = content
            .lines()
            .take(5)
            .any(|l| l.trim_start().starts_with("//!"));
        assert!(has_module_doc, "{} must begin with a `//!` module doc", rel);
    }
}

#[test]
fn test_guard_file_size_and_esloc_ceilings() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut violations = Vec::new();
    for dir_rel in &["src", "tests"] {
        for file in collect_rs_files(&manifest_dir.join(dir_rel)) {
            let rel = file
                .strip_prefix(manifest_dir)
                .unwrap_or(&file)
                .to_string_lossy();
            let content = fs::read_to_string(&file).expect("Failed to read source file");
            let (physical, esloc) = count_lines_and_esloc(&content);
            if physical >= MAX_PHYSICAL_LINES || esloc >= MAX_ESLOC {
                violations.push(format!(
                    "{} exceeds ceiling: physical={} (max <{}), eSLOC={} (max <{})",
                    rel, physical, MAX_PHYSICAL_LINES, esloc, MAX_ESLOC
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "GL-1015 File Size / eSLOC Ceiling Guard Failed:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_guard_analyzers_self_check() {
    assert!(!check_magic_numbers_in_source("bad.rs", "let h = 18.0;").is_empty());
    assert!(check_magic_numbers_in_source("ok.rs", "let mid = w * 0.5;").is_empty());
    assert!(!check_hardcoded_colors_in_source("bad.rs", "let c = Color32::WHITE;").is_empty());
    assert!(
        !check_hardcoded_colors_in_source("bad.rs", "let c = Color32::from_black_alpha(20);")
            .is_empty()
    );
    assert!(
        !check_token_arithmetic_cheating("bad.rs", "let w = theme.width.points() + 4.0;")
            .is_empty()
    );
    assert!(!check_domain_leakage_in_source("bad.rs", "struct PedigreeNode;").is_empty());
}
