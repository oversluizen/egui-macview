//! Golden Master Architecture Guardrail Suite for `egui-macview`.
//!
//! Normative References:
//! - GL-1005: Code for anything a machine could tell you got wrong.
//! - GL-1015: Industry Software Engineering & Testing Standards:
//!   - Regel 3: Zero Hardcoded Constants (100% Data-Driven from tokens/theme).
//!   - Regel 4: NASA/Google eSLOC Modularity Limit (< 400 eSLOC, < 599 physical lines).
//!   - Regel 10 / §3.3: Windows Desktop GUI Subsystem Invariant (#![cfg_attr(not(test), windows_subsystem = "windows")]).
//!   - Regel 11/12/13: Zero Hardcoded Colors, Dimensions & Domain Agnosticism.
//!
//! This suite serves as the Golden Master template across all ecosystem projects.

use std::fs;
use std::path::{Path, PathBuf};

/// Allowed dimensionless mathematical ratios and identity scalars.
const ALLOWED_RATIOS: &[f64] = &[0.0, 0.5, 1.0, 2.0];

/// Recursively collects all `.rs` files under a directory.
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
}

/// Computes executable Source Lines of Code (eSLOC) excluding comments and empty lines.
fn calculate_esloc(content: &str) -> usize {
    let mut esloc = 0;
    let mut in_block = false;
    for line in content.lines() {
        let mut trimmed = line.trim();
        if in_block {
            if let Some(pos) = trimmed.find("*/") {
                in_block = false;
                trimmed = trimmed[pos + 2..].trim();
            } else {
                continue;
            }
        }
        while let Some(pos) = trimmed.find("/*") {
            if let Some(end) = trimmed[pos + 2..].find("*/") {
                trimmed = trimmed[pos + 2 + end + 2..].trim();
            } else {
                in_block = true;
                trimmed = trimmed[..pos].trim();
                break;
            }
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            esloc += 1;
        }
    }
    esloc
}

/// Strips comments (`//...` and `/*...*/`) and string literals (`"..."`) from source code,
/// preserving newlines and line indices for accurate violation reporting.
fn strip_comments_and_strings(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut idx = 0;
    let mut in_block_comment = 0;

    while idx < len {
        if in_block_comment > 0 {
            if idx + 1 < len && chars[idx] == '*' && chars[idx + 1] == '/' {
                in_block_comment -= 1;
                idx += 2;
            } else {
                if chars[idx] == '\n' {
                    result.push('\n');
                } else {
                    result.push(' ');
                }
                idx += 1;
            }
            continue;
        }

        // Line comment
        if idx + 1 < len && chars[idx] == '/' && chars[idx + 1] == '/' {
            while idx < len && chars[idx] != '\n' {
                result.push(' ');
                idx += 1;
            }
            continue;
        }

        // Block comment start
        if idx + 1 < len && chars[idx] == '/' && chars[idx + 1] == '*' {
            in_block_comment += 1;
            result.push(' ');
            result.push(' ');
            idx += 2;
            continue;
        }

        // String literal
        if chars[idx] == '"' {
            result.push(' ');
            idx += 1;
            while idx < len && chars[idx] != '"' {
                if chars[idx] == '\\' && idx + 1 < len {
                    result.push(' ');
                    result.push(' ');
                    idx += 2;
                } else {
                    if chars[idx] == '\n' {
                        result.push('\n');
                    } else {
                        result.push(' ');
                    }
                    idx += 1;
                }
            }
            if idx < len && chars[idx] == '"' {
                result.push(' ');
                idx += 1;
            }
            continue;
        }

        result.push(chars[idx]);
        idx += 1;
    }

    result
}

/// Extracts all floating point literals from stripped source line.
fn extract_float_literals(line: &str) -> Vec<f64> {
    let mut floats = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut idx = 0;

    while idx < len {
        // Look for digit
        if chars[idx].is_ascii_digit() {
            // Check preceding character: must not be identifier char or dot
            if idx > 0
                && (chars[idx - 1].is_alphanumeric()
                    || chars[idx - 1] == '_'
                    || chars[idx - 1] == '.')
            {
                idx += 1;
                continue;
            }

            let start = idx;
            while idx < len && chars[idx].is_ascii_digit() {
                idx += 1;
            }

            // Check if dot follows
            if idx < len && chars[idx] == '.' {
                // Must not be range `..`
                if idx + 1 < len && chars[idx + 1] == '.' {
                    idx += 1;
                    continue;
                }
                // Must be followed by digit
                if idx + 1 < len && chars[idx + 1].is_ascii_digit() {
                    idx += 1; // consume dot
                    while idx < len && chars[idx].is_ascii_digit() {
                        idx += 1;
                    }
                    // Optional float suffix (f32, f64, _f32, _f64)
                    let num_str: String = chars[start..idx].iter().collect();
                    if let Ok(val) = num_str.parse::<f64>() {
                        floats.push(val);
                    }
                }
            }
        }
        idx += 1;
    }

    floats
}

// ============================================================================
// Guardrail 1: Zero Magic Numbers & Hardcoded Geometry (GL-1015 Regel 3)
// ============================================================================

#[test]
fn test_guard_zero_magic_numbers_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dirs = [
        manifest_dir.join("src").join("controls"),
        manifest_dir.join("src").join("layout"),
        manifest_dir.join("src").join("surfaces"),
    ];

    let mut files = Vec::new();
    for dir in &target_dirs {
        collect_rs_files(dir, &mut files);
    }
    assert!(
        !files.is_empty(),
        "Architecture Guard Error: Found zero source files in UI directories!"
    );

    let mut violations = Vec::new();

    for file_path in &files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let stripped = strip_comments_and_strings(&content);
        for (line_idx, line) in stripped.lines().enumerate() {
            let line_num = line_idx + 1;
            let floats = extract_float_literals(line);
            for val in floats {
                let is_allowed = ALLOWED_RATIOS
                    .iter()
                    .any(|&ratio| (val - ratio).abs() < 1e-6);

                if !is_allowed {
                    violations.push(format!(
                        "  - {relative_path}:{line_num}: Disallowed numeric literal `{val}` found. \
                         All geometry dimensions must resolve via `theme.*()` or `tokens.*()`. \
                         Allowed ratios: {ALLOWED_RATIOS:?}"
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] GL-1015 Regel 3 (Zero Magic Numbers):\n\
             Found {} hardcoded numeric literals in UI components:\n{}\n\
             Remediation: Extract dimensions into `MacTheme` or W3C DTCG `design_tokens.json`.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 2: Zero Hardcoded Colors (GL-1015 Regel 12)
// ============================================================================

#[test]
fn test_guard_zero_hardcoded_colors_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dirs = [
        manifest_dir.join("src").join("controls"),
        manifest_dir.join("src").join("layout"),
        manifest_dir.join("src").join("surfaces"),
    ];

    let mut files = Vec::new();
    for dir in &target_dirs {
        collect_rs_files(dir, &mut files);
    }

    let mut violations = Vec::new();

    for file_path in &files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        for (line_idx, original_line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            // Ignore line comments
            let code_part = original_line.split("//").next().unwrap_or("").trim();
            if code_part.is_empty() {
                continue;
            }

            // Forbidden inline color constructors
            for forbidden_pattern in [
                "Color32::from_rgb(",
                "Color32::from_rgba_premultiplied(",
                "Color32::from_rgba_unmultiplied(",
            ] {
                if code_part.contains(forbidden_pattern) {
                    violations.push(format!(
                        "  - {relative_path}:{line_num}: Inline color constructor `{forbidden_pattern}` forbidden. \
                         All colors must resolve via `theme.*()` or `tokens.*()`."
                    ));
                }
            }

            // Forbidden raw hex color codes in string literals
            if code_part.contains('"') {
                for part in code_part.split('"') {
                    if part.starts_with('#')
                        && (part.len() == 4 || part.len() == 7 || part.len() == 9)
                        && part[1..].chars().all(|c| c.is_ascii_hexdigit())
                    {
                        violations.push(format!(
                            "  - {relative_path}:{line_num}: Hardcoded hex color `{part}` forbidden in UI components. \
                             Colors must resolve via `MacTheme` or DTCG token catalog."
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] GL-1015 Regel 12 (Zero Hardcoded Colors):\n\
             Found {} hardcoded color violations in UI components:\n{}\n\
             Remediation: All colors must be sourced from `MacTheme` dynamic appearance resolvers.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 3: Modularity Limits (< 400 eSLOC & < 599 Physical Lines) (GL-1015 Regel 4)
// ============================================================================

#[test]
fn test_guard_modularity_limits_gl1015() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);
    assert!(
        !files.is_empty(),
        "Architecture Guard Error: Found zero source files in src/!"
    );

    let mut violations = Vec::new();

    for file_path in &files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let physical_lines = content.lines().count();
        let esloc = calculate_esloc(&content);

        // GL-1015 Hard Ceiling: < 400 eSLOC and < 599 physical lines
        if physical_lines >= 599 {
            violations.push(format!(
                "  - {relative_path}: {physical_lines} physical lines exceeds GL-1015 hard ceiling (max 598 lines)."
            ));
        }

        if esloc >= 400 {
            violations.push(format!(
                "  - {relative_path}: {esloc} eSLOC exceeds GL-1015 modularity limit (max 399 eSLOC)."
            ));
        }

        // Anti-Cramming Rule (Regel 19): No multi-statement cramming to fake LOC reduction
        for (line_idx, line) in content.lines().enumerate() {
            let code_part = line.split("//").next().unwrap_or("").trim();
            if !code_part.contains('"') && !code_part.ends_with('\\') {
                let without_arrays = code_part
                    .replace("; 2]", "]")
                    .replace("; 3]", "]")
                    .replace("; 4]", "]");
                if without_arrays.matches(';').count() >= 3 {
                    violations.push(format!(
                        "  - {relative_path}:{}: Multi-statement single-line cramming forbidden (>= 3 semicolons): `{code_part}`",
                        line_idx + 1
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] GL-1015 Regel 4 (Modularity & NASA/Google eSLOC):\n\
             Found {} modularity ceiling violations:\n{}\n\
             Remediation: Refactor large modules into submodules following the Single Responsibility Principle.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 4: Domain Agnosticism (Zero Domain Leakage into AppKit Primitives)
// ============================================================================

#[test]
fn test_guard_domain_agnosticism() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files);

    // Domain-specific terms from font editing, CAD, or external domain layers
    // that have zero business existing inside a generic AppKit GUI engine.
    let forbidden_domain_words = [
        "glyph",
        "kurbo",
        "bezpath",
        "patrix",
        "glif",
        "ufo",
        "designspace",
    ];

    let mut violations = Vec::new();

    for file_path in &files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        for (line_idx, line) in content.lines().enumerate() {
            let line_num = line_idx + 1;
            let lower = line.to_lowercase();
            for word in &forbidden_domain_words {
                if lower.contains(word) {
                    violations.push(format!(
                        "  - {relative_path}:{line_num}: Domain-specific term `{word}` forbidden in egui-macview: `{line}`"
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] Domain Agnosticism:\n\
             Found {} domain-specific terms leaked into egui-macview:\n{}\n\
             Remediation: egui-macview is a domain-agnostic AppKit layout and primitive suite. \
             Domain-specific models and terms belong in application crates, never in the GUI primitives.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 5: Zero Token Math Cheats & Artificial Arithmetic (GL-1015)
// ============================================================================

#[test]
fn test_guard_zero_token_math_cheats_in_ui_components() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let target_dirs = [
        manifest_dir.join("src").join("controls"),
        manifest_dir.join("src").join("layout"),
        manifest_dir.join("src").join("surfaces"),
    ];

    let mut files = Vec::new();
    for dir in &target_dirs {
        collect_rs_files(dir, &mut files);
    }

    let mut violations = Vec::new();
    let forbidden_patterns = [
        (
            "half * 0.5",
            "Synthesizing chevron depth via intermediate variable halving",
        ),
        (
            "* 0.5 * 0.5",
            "Chained halving to synthesize sub-element padding/size",
        ),
        (
            "* 0.25",
            "Fractional quarter-scaling to synthesize sub-element size",
        ),
        (
            "* 0.75",
            "Fractional three-quarter scaling to synthesize sub-element size",
        ),
        ("* 0.33", "Fractional third-scaling to synthesize dimension"),
        ("* 0.2", "Fractional fifth-scaling to synthesize dimension"),
        (
            ".height.points() *",
            "Synthesizing dimension from height metric",
        ),
        (
            ".width.points() *",
            "Synthesizing dimension from width metric",
        ),
        (
            ".size.points() *",
            "Synthesizing dimension from size metric",
        ),
        (
            "edge_inset * 2.0 - scrollbar_thickness",
            "Synthesizing scrollbar hover hit zone from edge_inset arithmetic",
        ),
    ];

    for file_path in &files {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let stripped = strip_comments_and_strings(&content);
        for (line_idx, line) in stripped.lines().enumerate() {
            let line_num = line_idx + 1;
            let compacted: String = line.chars().filter(|c| !c.is_whitespace()).collect();

            for (pattern, reason) in &forbidden_patterns {
                let pattern_compact: String =
                    pattern.chars().filter(|c| !c.is_whitespace()).collect();
                if compacted.contains(&pattern_compact) {
                    violations.push(format!(
                        "  - {relative_path}:{line_num}: Token math cheat `{pattern}` detected ({reason}). \
                         Must declare dedicated W3C DTCG token instead of arithmetic synthesis."
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] Zero Token Math Cheats (GL-1015):\n\
             Found {} token math cheat violations in UI components:\n{}\n\
             Remediation: Add explicit metrics to tokens/design_tokens.json and MacTheme.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 6: Zero Cross-Component Token Borrowing (GL-1015 SRP / SSOT)
// ============================================================================

#[test]
fn test_guard_zero_cross_component_token_borrowing() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let controls_dir = manifest_dir.join("src").join("controls");

    let mut files = Vec::new();
    collect_rs_files(&controls_dir, &mut files);

    let all_control_metric_methods = [
        "button",
        "color_well",
        "outline_view",
        "popover_metrics",
        "popup_button",
        "scroll_view",
        "search_field",
        "segmented",
        "slider",
        "split_view",
        "stack_view",
        "stepper",
        "thumbnail",
        "toggle",
    ];

    let mut violations = Vec::new();

    for file_path in &files {
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "mod.rs" {
            continue;
        }

        let allowed_metric_methods: &[&str] = match file_name {
            "button_cell.rs" => &["button"],
            "color_well.rs" => &["color_well"],
            "input_cell.rs" => &[],
            "outline_view.rs" => &["outline_view", "thumbnail"],
            "popup_button.rs" => &["popup_button"],
            "search_field.rs" => &["search_field"],
            "segmented.rs" => &["segmented"],
            "slider.rs" => &["slider"],
            "stepper.rs" => &["stepper"],
            "thumbnail_cell.rs" => &["thumbnail"],
            "toggle.rs" => &["toggle"],
            _ => &[],
        };

        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        let stripped = strip_comments_and_strings(&content);
        for (line_idx, line) in stripped.lines().enumerate() {
            let line_num = line_idx + 1;
            for &method in &all_control_metric_methods {
                if !allowed_metric_methods.contains(&method) {
                    let call_pattern = format!("theme.{method}(");
                    if line.contains(&call_pattern) {
                        violations.push(format!(
                            "  - {relative_path}:{line_num}: Cross-component token borrowing forbidden. \
                             Control `{file_name}` calls foreign metric `{call_pattern}`."
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] Zero Cross-Component Token Borrowing:\n\
             Found {} unauthorized token borrowing violations in src/controls/:\n{}\n\
             Remediation: Controls must only access their own designated metrics in MacTheme.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

// ============================================================================
// Guardrail 7: Windows Desktop GUI Subsystem Invariant (GL-1015 Regel 10 & §3.3)
// ============================================================================

#[test]
fn test_guard_windows_gui_subsystem_attribute() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_bin = manifest_dir.join("src").join("bin");
    let examples = manifest_dir.join("examples");
    let src_main = manifest_dir.join("src").join("main.rs");

    let mut entrypoints = Vec::new();
    if src_main.exists() {
        entrypoints.push(src_main);
    }
    if src_bin.exists() {
        collect_rs_files(&src_bin, &mut entrypoints);
    }
    if examples.exists() {
        collect_rs_files(&examples, &mut entrypoints);
    }

    let required_attr = "#![cfg_attr(not(test), windows_subsystem = \"windows\")]";
    let mut violations = Vec::new();

    for file_path in &entrypoints {
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));
        let relative_path = file_path
            .strip_prefix(manifest_dir)
            .unwrap_or(file_path)
            .to_string_lossy();

        // Check if this binary/example is a GUI entrypoint (depends on or uses eframe/winit/gui frameworks)
        let is_gui = content.contains("eframe")
            || content.contains("winit")
            || content.contains("run_native")
            || content.contains("iced")
            || content.contains("slint")
            || content.contains("tauri");

        if is_gui && !content.contains(required_attr) {
            if content.contains("windows_subsystem") {
                violations.push(format!(
                    "  - {relative_path}: Incorrect windows_subsystem declaration. \
                     MUST be `#![cfg_attr(not(test), windows_subsystem = \"windows\")]` to avoid console popups in debug builds."
                ));
            } else {
                violations.push(format!(
                    "  - {relative_path}: Missing mandatory Windows GUI Subsystem attribute `{required_attr}`. \
                     GL-1015 §3.3 mandates all GUI binaries and examples suppress attached console allocation on Windows."
                ));
            }
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[FATAL ARCHITECTURE GUARD VIOLATION] GL-1015 §3.3 (Windows Desktop GUI Subsystem Wet):\n\
             Found {} GUI entrypoint(s) missing or violating `#![cfg_attr(not(test), windows_subsystem = \"windows\")]`:\n{}\n\
             Remediation: Add `#![cfg_attr(not(test), windows_subsystem = \"windows\")]` at the crate root of each GUI binary or example.\n",
            violations.len(),
            violations.join("\n")
        );
    }
}

