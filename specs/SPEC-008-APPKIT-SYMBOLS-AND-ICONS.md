# SPEC-008A: AppKit Vector Symbols, Iconography & CupertinoIcons Engine

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-008A |
| **Document Title** | AppKit Vector Symbols, Iconography & CupertinoIcons Engine |
| **Document Version** | `0.4.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.4.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/symbol_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Initial Icon Exploration: Unicode Character Mappings |
| `0.2.0` | 2026-11-06 | Architecture Team | Problem Identification: Windows Segoe UI Emoji Multi-Color Divergence in `macview_gallery` |
| `0.3.0` | 2026-11-06 | Architecture Team | Normative Release: CupertinoIcons Vector Font Engine (`MacSymbol`, `setup_symbols_font`, Anti-Emoji Guard) |
| `0.4.0` | 2026-11-07 | Architecture Team | Milestone 0.4.0 Release: Full AppKit Vector Symbol suite synchronized with Crate v0.4.0 |
| `0.4.1` | 2026-11-08 | Architecture Team | Consolidation Hardening: Disambiguated Document ID (`SPEC-008A`) and menu checkmark vector rendering |

---

## 1. Scope & System Architecture

### 1.1 Problem Statement & Normative Justification
Native macOS desktop applications rely on SF Symbols and AppKit iconography that are:
1. **100% Monochrome & Tintable:** Symbols rasterize using the active system foreground fill or label color (`text_primary`, `text_secondary`, `selected_content_text_color`).
2. **Optically Scaled & Balanced:** Glyphs adhere to strict baseline alignment, matching cap heights and optical weights of San Francisco Pro text.
3. **Cross-Platform Invariance:** On non-macOS hosts (particularly Windows 11), using standard Unicode emoji (such as `"🔍"` or `"⚙"`) causes immediate fallback to platform emoji fonts (e.g., `Segoe UI Emoji`), rendering multi-colored, cartoonish icons that violate Apple HIG and disrupt professional contrast ratios.

### 1.2 Normative Solution: CupertinoIcons Engine
`egui-macview` embeds the industry-standard Flutter `CupertinoIcons` TrueType font (licensed under MIT). The vector glyphs are mapped to Unicode Private Use Area (PUA `0xF200..0xF8FF`) codepoints, ensuring:
- Zero color contamination from host emoji engines.
- Crisp vector rendering at any DPI scale factor.
- Seamless fallback integration within `egui::FontDefinitions`.

---

## 2. `MacSymbol` Enumeration & Mapping Specification

### 2.1 Enum Definition & Codepoints
The crate SHALL expose the `MacSymbol` enum containing exactly the following variants and corresponding Unicode Private Use Area (PUA) scalar values:

| Variant | SF Symbols Parity | Cupertino Unicode Scalar | Hex Codepoint |
| :--- | :--- | :--- | :--- |
| `Search` | `magnifyingglass` | `\u{f4a5}` | `0xF4A5` |
| `ChevronRight` | `chevron.right` | `\u{f3d3}` | `0xF3D3` |
| `ChevronLeft` | `chevron.left` | `\u{f3d2}` | `0xF3D2` |
| `ChevronDown` | `chevron.down` | `\u{f5d5}` | `0xF5D5` |
| `ChevronUp` | `chevron.up` | `\u{f5e5}` | `0xF5E5` |
| `SidebarLeading` | `sidebar.leading` / `sidebar.left` | `\u{f7d6}` | `0xF7D6` |
| `Gear` | `gearshape` / `gear` | `\u{f43c}` | `0xF43C` |
| `Sliders` | `slider.horizontal.3` | `\u{f7dc}` | `0xF7DC` |
| `Checkmark` | `checkmark` | `\u{f3fd}` | `0xF3FD` |
| `Xmark` | `xmark` | `\u{f404}` | `0xF404` |
| `Plus` | `plus` | `\u{f489}` | `0xF489` |
| `Minus` | `minus` | `\u{f70f}` | `0xF70F` |
| `ArrowClockwise` | `arrow.clockwise` | `\u{f49a}` | `0xF49A` |
| `ArrowCounterClockwise` | `arrow.counterclockwise` | `\u{f21c}` | `0xF21C` |

### 2.2 Method Signatures & Trait Implementations
The `MacSymbol` type SHALL provide:
```rust
impl MacSymbol {
    /// Returns the character glyph representing the symbol in the CupertinoIcons font.
    pub const fn as_char(&self) -> char;

    /// Returns the string representation of the symbol glyph.
    pub fn as_str(&self) -> &str;

    /// Returns the 32-bit Unicode scalar codepoint of the symbol.
    pub const fn codepoint(&self) -> u32;
}

impl std::fmt::Display for MacSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl AsRef<str> for MacSymbol {
    fn as_ref(&self) -> &str;
}
```

---

## 3. Font Asset & Registration Specification

### 3.1 Embedded TrueType Asset
The crate SHALL embed the `CupertinoIcons.ttf` font asset via `include_bytes!` at compile time:
```rust
pub const CUPERTINO_ICONS_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/CupertinoIcons.ttf");
```
- Asset location: `assets/fonts/CupertinoIcons.ttf`
- License: MIT License (Flutter Authors)
- Minimum file size: > 100,000 bytes.
- Integrity: Valid TrueType font file header (`0x00, 0x01, 0x00, 0x00`).

### 3.2 Font Registration Function
The crate SHALL provide the public function:
```rust
pub fn setup_symbols_font(fonts: &mut egui::FontDefinitions);
```
Normative behavior of `setup_symbols_font`:
1. It MUST insert `CUPERTINO_ICONS_FONT_BYTES` into `fonts.font_data` under the key `"cupertino_icons"`.
2. It MUST append `"cupertino_icons"` as a fallback font in `egui::FontFamily::Proportional`.
3. It MUST append `"cupertino_icons"` as a fallback font in `egui::FontFamily::Monospace`.
4. Fallback order ensures standard alphanumeric text glyphs resolve from the primary user font (e.g., SF Pro, Inter, Segoe UI), while PUA icon codepoints fall through to `CupertinoIcons`.

---

## 4. Verification Oracles & Anti-Emoji Guard

The verification suite (`tests/symbol_tests.rs`) SHALL enforce:
1. **Asset Integrity Oracle:** Embedded font byte array is non-empty, matches valid TrueType magic bytes, and parses successfully.
2. **Glyph Mapping Oracle:** All 14 variants of `MacSymbol` map to their exact assigned PUA scalar values and format to non-empty strings.
3. **Anti-Emoji Guard Oracle:** Static analysis assertion verifying that `examples/macview_gallery.rs` contains zero occurrences of the `"🔍"` emoji, preventing regression to platform-dependent emoji rendering.
