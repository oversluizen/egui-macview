//! Static analysis helper functions for architecture fitness guards.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const ALLOWED_RATIOS: &[f64] = &[0.0, 0.5, 1.0, 2.0];
pub const FORBIDDEN_DOMAIN_TERMS: &[&str] = &[
    "pedigree", "gedcom", "kinship", "ancestor", "proband", "marriage", "ahnen", "stamboom",
    "divorce", "adoptive", "foster", "sibling",
];
pub const MAX_PHYSICAL_LINES: usize = 599;
pub const MAX_ESLOC: usize = 400;

pub fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
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
    let (mut i, n) = (0, chars.len());
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
            let (mut hashes, mut j) = (0usize, i + 1);
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
        let (line_num, bytes, len, mut i) = (line_idx + 1, line.as_bytes(), line.len(), 0);
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
                if i + 1 < len && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                        i += 1;
                    }
                    let num_end = i;
                    if i + 3 <= len && (&line[i..i + 3] == "f32" || &line[i..i + 3] == "f64") {
                        i += 3;
                    }
                    if let Ok(val) = line[start..num_end].replace('_', "").parse::<f64>() {
                        results.push((line_num, line[start..i].to_string(), val));
                    }
                    continue;
                }
                if i + 3 <= len && (&line[i..i + 3] == "f32" || &line[i..i + 3] == "f64") {
                    let num_end = i;
                    i += 3;
                    if let Ok(val) = line[start..num_end].replace('_', "").parse::<f64>() {
                        results.push((line_num, line[start..i].to_string(), val));
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
    extract_float_literals(&stripped)
        .into_iter()
        .filter(|(_, _, val)| !ALLOWED_RATIOS.iter().any(|&a| (val - a).abs() < 1e-9))
        .map(|(line_num, raw, val)| {
            format!("{file_label}:{line_num} -> forbidden magic float literal `{raw}` ({val})")
        })
        .collect()
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
        for call in &forbidden_calls {
            if line.contains(call) {
                violations.push(format!("{file_label}:{} -> `{call}`", idx + 1));
            }
        }
    }
    for (idx, raw_line) in source.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        if let Some(pos) = raw_line.find("\"#") {
            let after = &raw_line[pos + 2..];
            let hex_len = after.chars().take_while(|c| c.is_ascii_hexdigit()).count();
            if (hex_len == 6 || hex_len == 8) && after.as_bytes().get(hex_len) == Some(&b'"') {
                violations.push(format!(
                    "{file_label}:{} -> inline hex `\"#{}\"`",
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
        if let Some(pos) = line.find(".points()") {
            let after = line[pos + ".points()".len()..].trim_start();
            if after.starts_with('+') || after.starts_with('-') {
                let expr_rest = after[1..].trim_start();
                for (_, raw, val) in extract_float_literals(expr_rest) {
                    let is_allowed = ALLOWED_RATIOS.iter().any(|&a| (val - a).abs() < 1e-9);
                    if !is_allowed && expr_rest.starts_with(&raw) {
                        violations.push(format!(
                            "{file_label}:{} -> `.points() {} {raw}`",
                            idx + 1,
                            &after[..1]
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
                violations.push(format!("{file_label}:{} -> `{term}`", idx + 1));
            }
        }
    }
    violations
}

pub fn check_uibuilder_id_salt_in_source(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    let (mut violations, needle, mut from) = (Vec::new(), "UiBuilder::new()", 0);
    while let Some(rel) = stripped[from..].find(needle) {
        let abs = from + rel;
        let line_num = stripped[..abs].lines().count().max(1);
        let after = &stripped[abs + needle.len()..];
        let span = after
            .find(';')
            .unwrap_or(after.len())
            .min(after.find(needle).unwrap_or(after.len()))
            .min(300);
        if !after[..span].contains(".id_salt(") {
            violations.push(format!(
                "{file_label}:{line_num} -> `UiBuilder::new()` missing `.id_salt(...)`"
            ));
        }
        from = abs + needle.len();
    }
    violations
}

/// Verifies Windows GUI Subsystem & Safe `eframe` Startup invariants on GUI entrypoints.
pub fn check_gui_entrypoint_safety(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    if !stripped.contains("fn main") {
        return Vec::new();
    }
    let mut violations = Vec::new();
    if !source.contains("#![cfg_attr(not(test), windows_subsystem = \"windows\")]") {
        violations.push(format!(
            "{file_label} -> missing `#![cfg_attr(not(test), windows_subsystem = \"windows\")]`"
        ));
    }
    if stripped.contains("eframe::run_native") || stripped.contains("NativeOptions") {
        if stripped.contains("NativeOptions::default()") {
            violations.push(format!(
                "{file_label} -> bare `NativeOptions::default()` is forbidden (Safe eframe Startup Guard)"
            ));
        }
        for required in [
            "Renderer::Glow",
            "InstanceFlags::empty()",
            "system_font_fallback: false",
            "persist_window: false",
        ] {
            if !stripped.contains(required) {
                violations.push(format!(
                    "{file_label} -> `eframe::run_native` missing safe startup setting `{required}`"
                ));
            }
        }
    }
    violations
}

/// Forbids `allow(dead_code)` in `src/` production modules (Anti-Spookcode Guard).
pub fn check_anti_spookcode_in_source(file_label: &str, source: &str) -> Vec<String> {
    let stripped = strip_comments_and_strings(source);
    let mut violations = Vec::new();
    for (idx, line) in stripped.lines().enumerate() {
        let compact: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        if compact.contains("allow(dead_code)") {
            violations.push(format!(
                "{file_label}:{} -> forbidden `allow(dead_code)` in production `src/`",
                idx + 1
            ));
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

pub fn collect_json_leaf_paths(
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
            let next = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            let track = is_in_metrics_or_color
                || next.starts_with("macview.metrics")
                || next.starts_with("macview.color")
                || next.starts_with("macview.motion")
                || next.starts_with("macview.typography.tracking_table");
            collect_json_leaf_paths(v, next, track, out);
        }
    }
}
