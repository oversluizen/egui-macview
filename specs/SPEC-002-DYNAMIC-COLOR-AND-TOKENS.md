# SPEC-002A: Dynamic Color, Tokens & MacBackgroundStyle Contrast Engine

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-002A |
| **Document Title** | Dynamic Color, Tokens & MacBackgroundStyle Contrast Engine |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/wcag_contrast_and_visuals_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs`, `tests/cots_red_baseline_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, W3C WCAG 2.1 §1.4.3, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: W3C DTCG Token Catalog & Color Parsers |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: MacBackgroundStyle Dynamic Contrast Engine & WCAG 2.1 AA Luminance Formulations |
| `0.4.1` | 2026-11-08 | Architecture Team | Consolidation Hardening: Disambiguated Document ID (`SPEC-002A`), unmultiplied RGBA DTCG parsing, and context-stored `MacTheme::from_ctx` |

---

## 1. Scope & Normative Foundation

This specification governs the dynamic color space resolution, design token mappings, accessibility contrast requirements, and mathematical background style classification for `egui-macview`:
- **W3C Design Tokens Community Group (DTCG):** Dynamic semantic color mappings defined in `tokens/design_tokens.json`.
- **W3C WCAG 2.1 §1.4.3 (Level AA):** Strict minimum contrast ratio requirement of $\ge 4.5:1$ for standard text and $\ge 3.0:1$ for large text across both `Aqua` and `DarkAqua` appearances.
- **Apple AppKit `NSBackgroundStyle` Fidelity:** Deterministic derivation of `MacBackgroundStyle` (Light vs Dark) ensuring foreground readability on dynamic accent colors, selection fills, and container surfaces.

---

## 2. Dynamic Color Space & Token Architecture

Dynamic theme colors are resolved through `MacTheme::colors()`, mapping semantic AppKit names to sRGB hexadecimal definitions in `design_tokens.json`:
- `window_background`: Aqua `#ECECEC`, DarkAqua `#323232`.
- `control_background`: Aqua `#FFFFFF`, DarkAqua `#1E1E1E`.
- `control_accent`: Aqua `#007AFF`, DarkAqua `#0A84FF`.
- `selected_control`: Aqua `#007AFF`, DarkAqua `#0A84FF`.
- `label`: Aqua `#000000`, DarkAqua `#DCDCDC`.
- `secondary_label`: Aqua `rgba(0, 0, 0, 0.55)`, DarkAqua `#A0A0A5`.

---

## 3. WCAG 2.1 AA Relative Luminance Formula

For any sRGB color with channel values $R, G, B \in [0, 255]$, each channel is converted to linear space:

$$R_{\text{srgb}} = \frac{R}{255}, \quad R_{\text{linear}} = \begin{cases} \frac{R_{\text{srgb}}}{12.92} & \text{if } R_{\text{srgb}} \le 0.04045 \\ \left( \frac{R_{\text{srgb}} + 0.055}{1.055} \right)^{2.4} & \text{otherwise} \end{cases}$$

The relative luminance $L$ of the color is computed in accordance with WCAG 2.1 §1.4.3:

$$L = 0.2126 \cdot R_{\text{linear}} + 0.7152 \cdot G_{\text{linear}} + 0.0722 \cdot B_{\text{linear}}$$

The contrast ratio between two relative luminance values $L_1$ (lighter) and $L_2$ (darker) is:

$$\text{Contrast Ratio} = \frac{L_1 + 0.05}{L_2 + 0.05}$$

---

## 4. `MacBackgroundStyle` Mathematical Specification

In macOS AppKit, controls render content differently depending on whether the enclosing background is light or dark. In `egui-macview`, this is governed by `MacBackgroundStyle`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacBackgroundStyle {
    /// Background is light; foreground text and glyphs MUST be dark (minimum 4.5:1 contrast).
    #[default]
    Light,
    /// Background is dark; foreground text and glyphs MUST be light / on-accent white (minimum 4.5:1 contrast).
    Dark,
}
```

### 4.1 Crossover Threshold Derivation

To guarantee a minimum 4.5:1 contrast ratio against the background under WCAG 2.1 Level AA:

1. **Light Background Style (requiring Dark Text $L_{\text{dark}} \approx 0.0$):**
   $$\text{Contrast}_{\text{dark}} = \frac{L + 0.05}{0.0 + 0.05} = 20 L + 1.0 \ge 4.5 \iff 20 L \ge 3.5 \iff L \ge 0.175$$

2. **Dark Background Style (requiring On-Accent White Text $L_{\text{white}} = 1.0$):**
   $$\text{Contrast}_{\text{white}} = \frac{1.0 + 0.05}{L + 0.05} = \frac{1.05}{L + 0.05} \ge 4.5 \iff 1.05 \ge 4.5(L + 0.05) \iff L \le \frac{1.05}{4.5} - 0.05 \approx 0.1833$$

### 4.2 Decision Rule

The canonical crossover threshold is mathematically fixed at $L_{\text{threshold}} = 0.179$:

$$\text{MacBackgroundStyle}(L) = \begin{cases} \text{MacBackgroundStyle::Dark} \implies \text{On-Accent White Text } (L_w = 1.0) & \text{if } L \le 0.179 \\ \text{MacBackgroundStyle::Light} \implies \text{Dark / Near-Black Text } (L_b \approx 0.0) & \text{if } L > 0.179 \end{cases}$$

- When $L \le 0.179$: Contrast against white is $\frac{1.05}{0.179 + 0.05} = \frac{1.05}{0.229} \approx 4.585 \ge 4.5:1$.
- When $L > 0.179$: Contrast against black is $\frac{L + 0.05}{0.05} = 20 L + 1.0 > 20(0.179) + 1.0 = 4.58 \ge 4.5:1$.

This mathematical boundary guarantees that the contrast ratio between the foreground text/icon and the underlying background fill is strictly $\ge 4.5:1$ across all custom accents, system highlights, and container surfaces.

---

## 5. Verification & Test Oracles

Automated test suites in `tests/wcag_contrast_and_visuals_tests.rs` verify:
1. `Aqua` and `DarkAqua` labels against `window_background` ($\ge 4.5:1$).
2. Secondary labels and section headers against panel backgrounds ($\ge 4.5:1$).
3. Synchronization of `egui::Visuals` with dark mode to ensure host widget compliance.
