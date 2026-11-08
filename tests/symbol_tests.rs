//! Normative symbol & iconography test suite for `egui-macview` (SPEC-008).
//!
//! Enforces:
//! 1. Embedded TrueType font byte integrity.
//! 2. `MacSymbol` variant glyph codepoints in the Unicode Private Use Area (PUA).
//! 3. Deterministic `egui::FontDefinitions` fallback registration.
//! 4. Anti-Emoji Guard: verifies that production examples contain zero host emoji (such as "🔍").

use egui_macview::{setup_symbols_font, MacSymbol, CUPERTINO_ICONS_FONT_BYTES};

#[test]
fn test_font_byte_integrity() {
    // CupertinoIcons TrueType font must be embedded and non-empty.
    assert!(
        CUPERTINO_ICONS_FONT_BYTES.len() > 100_000,
        "Embedded CupertinoIcons font must be at least 100 KB, but got {} bytes",
        CUPERTINO_ICONS_FONT_BYTES.len()
    );

    // TrueType font magic bytes: 0x00, 0x01, 0x00, 0x00 (or 'OTTO' for OpenType)
    let magic = &CUPERTINO_ICONS_FONT_BYTES[0..4];
    assert_eq!(
        magic,
        &[0x00, 0x01, 0x00, 0x00],
        "Font must begin with TrueType magic header [0x00, 0x01, 0x00, 0x00]"
    );
}

#[test]
fn test_mac_symbol_glyph_mappings() {
    let test_cases: &[(MacSymbol, char, u32, &str)] = &[
        (MacSymbol::Search, '\u{f4a5}', 0xf4a5, "Search"),
        (MacSymbol::ChevronRight, '\u{f3d3}', 0xf3d3, "ChevronRight"),
        (MacSymbol::ChevronLeft, '\u{f3d2}', 0xf3d2, "ChevronLeft"),
        (MacSymbol::ChevronDown, '\u{f5d5}', 0xf5d5, "ChevronDown"),
        (MacSymbol::ChevronUp, '\u{f5e5}', 0xf5e5, "ChevronUp"),
        (
            MacSymbol::SidebarLeading,
            '\u{f7d6}',
            0xf7d6,
            "SidebarLeading",
        ),
        (MacSymbol::Gear, '\u{f43c}', 0xf43c, "Gear"),
        (MacSymbol::Sliders, '\u{f7dc}', 0xf7dc, "Sliders"),
        (MacSymbol::Checkmark, '\u{f3fd}', 0xf3fd, "Checkmark"),
        (MacSymbol::Xmark, '\u{f404}', 0xf404, "Xmark"),
        (MacSymbol::Plus, '\u{f489}', 0xf489, "Plus"),
        (MacSymbol::Minus, '\u{f70f}', 0xf70f, "Minus"),
        (
            MacSymbol::ArrowClockwise,
            '\u{f49a}',
            0xf49a,
            "ArrowClockwise",
        ),
        (
            MacSymbol::ArrowCounterClockwise,
            '\u{f21c}',
            0xf21c,
            "ArrowCounterClockwise",
        ),
    ];

    assert_eq!(test_cases.len(), 14, "Must cover all 14 MacSymbol variants");

    let mut seen_codepoints = std::collections::HashSet::new();

    for (symbol, expected_char, expected_codepoint, name) in test_cases {
        assert_eq!(
            symbol.as_char(),
            *expected_char,
            "Glyph char mismatch for {name}"
        );
        assert_eq!(
            symbol.codepoint(),
            *expected_codepoint,
            "Glyph codepoint mismatch for {name}"
        );
        assert_eq!(
            symbol.as_str(),
            expected_char.to_string().as_str(),
            "as_str mismatch for {name}"
        );
        assert_eq!(
            format!("{symbol}"),
            expected_char.to_string(),
            "Display format mismatch for {name}"
        );
        assert_eq!(
            symbol.as_ref(),
            expected_char.to_string().as_str(),
            "AsRef<str> mismatch for {name}"
        );

        // Invariant: All glyphs must reside in Unicode Private Use Area (PUA)
        assert!(
            (0xe000..=0xf8ff).contains(expected_codepoint),
            "Codepoint 0x{:04x} for {name} must reside in PUA range 0xE000..=0xF8FF",
            expected_codepoint
        );

        // Invariant: Uniqueness across all enum variants
        assert!(
            seen_codepoints.insert(*expected_codepoint),
            "Duplicate codepoint 0x{:04x} detected in variant {name}",
            expected_codepoint
        );
    }
}

#[test]
fn test_setup_symbols_font_registration() {
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);

    // Must install cupertino_icons font data
    assert!(
        fonts.font_data.contains_key("cupertino_icons"),
        "FontDefinitions must contain 'cupertino_icons' entry in font_data"
    );

    // Must install as fallback in Proportional family
    let prop_family = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .expect("Proportional family must exist");
    assert!(
        prop_family.iter().any(|f| f == "cupertino_icons"),
        "Proportional family must contain 'cupertino_icons' fallback"
    );

    // Must install as fallback in Monospace family
    let mono_family = fonts
        .families
        .get(&egui::FontFamily::Monospace)
        .expect("Monospace family must exist");
    assert!(
        mono_family.iter().any(|f| f == "cupertino_icons"),
        "Monospace family must contain 'cupertino_icons' fallback"
    );

    // Invariant: Idempotency (calling setup_symbols_font again does not duplicate entries)
    setup_symbols_font(&mut fonts);
    let prop_count = fonts.families[&egui::FontFamily::Proportional]
        .iter()
        .filter(|&f| f == "cupertino_icons")
        .count();
    let mono_count = fonts.families[&egui::FontFamily::Monospace]
        .iter()
        .filter(|&f| f == "cupertino_icons")
        .count();
    assert_eq!(
        prop_count, 1,
        "Idempotency violation: cupertino_icons duplicated in Proportional family"
    );
    assert_eq!(
        mono_count, 1,
        "Idempotency violation: cupertino_icons duplicated in Monospace family"
    );
}

#[test]
fn test_anti_emoji_guard_in_gallery() {
    let gallery_path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/macview_gallery.rs");
    let content = std::fs::read_to_string(gallery_path)
        .unwrap_or_else(|e| panic!("Failed to read {gallery_path}: {e}"));

    // Anti-Emoji Guard: Zero occurrences of Windows Segoe UI / Apple Color Emoji '🔍'
    let emoji = "🔍";
    let matches: Vec<(usize, &str)> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(emoji))
        .map(|(i, line)| (i + 1, line.trim()))
        .collect();

    assert!(
        matches.is_empty(),
        "Anti-Emoji Guard Violation: Found {} occurrence(s) of emoji '{emoji}' in macview_gallery.rs:\n{:#?}",
        matches.len(),
        matches
    );
}

#[test]
fn test_mac_symbol_const_and_all_variants() {
    // Compile-time constant evaluation verification
    const SEARCH_STR: &str = MacSymbol::Search.as_str();
    const SEARCH_CHAR: char = MacSymbol::Search.as_char();
    const SEARCH_CODEPOINT: u32 = MacSymbol::Search.codepoint();
    assert_eq!(SEARCH_STR, "\u{f4a5}");
    assert_eq!(SEARCH_CHAR, '\u{f4a5}');
    assert_eq!(SEARCH_CODEPOINT, 0xf4a5);

    // MacSymbol::ALL contains exactly 14 unique variants
    assert_eq!(
        MacSymbol::ALL.len(),
        14,
        "MacSymbol::ALL must contain all 14 variants"
    );
    let mut unique_set = std::collections::HashSet::new();
    for sym in &MacSymbol::ALL {
        assert!(
            unique_set.insert(*sym),
            "MacSymbol::ALL contains duplicate variant {sym:?}"
        );
    }
}

#[test]
fn test_symbols_headless_shaping_and_layout() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    setup_symbols_font(&mut fonts);
    ctx.set_fonts(fonts);

    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        for &sym in &MacSymbol::ALL {
            let font_id = egui::FontId::proportional(13.0);

            // 1. Isolated vector symbol shaping
            let solo_galley = ui.painter().layout_no_wrap(
                sym.as_str().to_owned(),
                font_id.clone(),
                egui::Color32::BLACK,
            );
            assert!(
                solo_galley.size().x > 0.0,
                "Symbol {sym:?} isolated layout must have positive width, got {:?}",
                solo_galley.size()
            );
            assert!(
                solo_galley.size().y > 0.0,
                "Symbol {sym:?} isolated layout must have positive height, got {:?}",
                solo_galley.size()
            );

            // 2. Mixed symbol + ASCII label shaping (e.g. "🔍 ⌘K")
            let mixed_str = format!("{} ⌘K", sym.as_str());
            let mixed_galley =
                ui.painter()
                    .layout_no_wrap(mixed_str, font_id, egui::Color32::BLACK);
            assert!(
                mixed_galley.size().x > solo_galley.size().x,
                "Mixed label for {sym:?} must be wider than isolated symbol"
            );
        }
    });
    out.textures_delta.clear();
}
