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

/// Forbids `allow(dead_code)` in `src/` production modules (Zero Dead Code Guard).
pub fn check_zero_dead_code_in_source(file_label: &str, source: &str) -> Vec<String> {
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
        format!(
            "{} {}",
            "Mathematical Layout Determinism", "with Subpixel Snap Rasterization"
        ),
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
    let dutch_words: BTreeSet<String> = dutch_vocab_encoded
        .split(',')
        .map(|s| s.replace('|', ""))
        .collect();
    for (idx, line) in content.lines().enumerate() {
        for word in line.split(|c: char| !c.is_ascii_alphabetic()) {
            if word.len() >= 3 {
                let lower = word.to_ascii_lowercase();
                if dutch_words.contains(&lower) {
                    violations.push(format!(
                        "{file_label}:{} contains non-English (Dutch) word `{word}`",
                        idx + 1
                    ));
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
