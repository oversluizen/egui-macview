# SPEC-001: Design Tokens, Optical Tracking & WCAG AA Contrast

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-001 |
| **Document Title** | Design Tokens, Optical Tracking & WCAG AA Contrast |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/wcag_contrast_and_visuals_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs`, `tests/architecture_guard_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, W3C WCAG 2.1 §1.4.3, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: W3C DTCG Token Catalog & Color Parsers |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Extended Control Metrics, WCAG 2.1 AA Contrast Enforcement, and Zero-Hardcoding Architecture Guards |

---

## 1. Scope & Normative Foundation

This specification governs the design token pipeline, color space resolution, typographic optical tracking, and accessibility contrast requirements for `egui-macview`:
- **W3C Design Tokens Community Group (DTCG):** 100% data-driven token definitions stored in `tokens/design_tokens.json` and `tokens/typography.json`.
- **Zero Hardcoding Invariant (GL-1015):** Direct numeric float literals (e.g. `18.0`, `22.0`, `44.0`) or hex color codes (`#FFFFFF`, `Color32::from_rgb`) are strictly forbidden in UI components.
- **W3C WCAG 2.1 §1.4.3 (Level AA):** Minimum contrast ratio of $\ge 4.5:1$ for normal text and $\ge 3.0:1$ for large text across both `Aqua` (light) and `DarkAqua` (dark) appearances.

---

## 2. W3C DTCG Token Architecture

All metrics and colors are loaded through `TokenCatalog` and parsed via `serde_json`:

```json
{
  "$name": "egui-macview Design Tokens",
  "$version": "0.2.0",
  "macview": {
    "metrics": {
      "control": {
        "focus_ring": {
          "stroke_width": { "$type": "dimension", "$value": "3.0pt" },
          "offset": { "$type": "dimension", "$value": "1.0pt" },
          "corner_radius_delta": { "$type": "dimension", "$value": "2.0pt" }
        }
      }
    }
  }
}
```

### 2.1 Control Metric Namespaces
- `button`: `regular.height` (22.0pt), `regular.corner_radius` (6.0pt).
- `text_field`: `compact_infobox.height` (18.0pt), `regular.height` (22.0pt), `regular.corner_radius` (4.0pt).
- `segmented`: `regular.height` (20.0pt), `regular.corner_radius` (5.0pt).
- `toggle`: `width` (26.0pt), `height` (15.0pt), `knob_size` (13.0pt).
- `slider`: `track_height` (3.0pt), `thumb_size` (16.0pt), `thumb_stroke_width` (1.0pt).
- `stepper`: `width` (14.0pt), `height` (22.0pt), `arrow_size` (6.0pt).
- `color_well`: `width` (44.0pt), `height` (23.0pt), `swatch_inset` (3.0pt).
- `search_field`: `height` (22.0pt), `corner_radius` (11.0pt), `icon_size` (12.0pt).
- `popup_button`: `height` (22.0pt), `corner_radius` (5.0pt), `chevron_size` (8.0pt).
- `outline_view`: `row_height` (24.0pt), `indent_step` (16.0pt), `chevron_size` (7.0pt).
- `thumbnail`: `size` (24.0pt), `corner_radius` (4.0pt), `border_width` (1.0pt).

---

## 3. Typographic Optical Tracking Function

Letter-spacing tracking across point sizes MUST be computed using cubic polynomial interpolation grounded in Apple Advanced Typography (AAT) specifications and resolved from token catalog `macview.typography.tracking_table`:

$$\text{Tracking}(p) = a \cdot p^3 + b \cdot p^2 + c \cdot p + d$$

- Token `typography.tracking_table.pt_11`: $11.0\,\text{pt} \implies -0.23\,\text{pt}$ (compact labels and input cells)
- Token `typography.tracking_table.pt_12`: $12.0\,\text{pt} \implies -0.15\,\text{pt}$ (outline views and secondary items)
- Token `typography.tracking_table.pt_13`: $13.0\,\text{pt} \implies -0.08\,\text{pt}$ (standard body and control labels)
- Token `typography.tracking_table.pt_17`: $17.0\,\text{pt} \implies 0.00\,\text{pt}$ (headings and titles)

---

## 4. WCAG 2.1 AA Contrast Enforcement

### 4.1 Relative Luminance Formula
For any sRGB color with channel values $R, G, B \in [0, 255]$:

$$R_{\text{srgb}} = \frac{R}{255}, \quad R_{\text{linear}} = \begin{cases} \frac{R_{\text{srgb}}}{12.92} & \text{if } R_{\text{srgb}} \le 0.04045 \\ \left( \frac{R_{\text{srgb}} + 0.055}{1.055} \right)^{2.4} & \text{otherwise} \end{cases}$$

Relative luminance $Y$:
$$Y = 0.2126 \cdot R_{\text{linear}} + 0.7152 \cdot G_{\text{linear}} + 0.0722 \cdot B_{\text{linear}}$$

Contrast ratio between lighter luminance $L_1$ and darker luminance $L_2$:
$$\text{Contrast Ratio} = \frac{L_1 + 0.05}{L_2 + 0.05}$$

### 4.2 Contrast Invariants
1. **Primary Text Labels (`theme.label()`):** $\ge 4.5:1$ against `window_background` and `control_background` in both `Aqua` and `DarkAqua`.
2. **Secondary / Header Text (`theme.secondary_label()`):** $\ge 4.5:1$ against inspector headers and container panels.
3. **Context Synchronization:** `MacTheme::apply_to_egui` MUST synchronize `egui::Visuals::light()` and `egui::Visuals::dark()` ensuring host text widgets automatically satisfy WCAG thresholds.

### 4.3 `MacBackgroundStyle` Dynamic Contrast Engine (Light vs Dark)

Controls, selection fills, and table rows dynamically resolve their foreground text and icon contrast against container or selection backgrounds via `MacBackgroundStyle` (`NSBackgroundStyle` AppKit equivalent):

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

#### 4.3.1 Mathematical Crossover Threshold Formulation
To strictly satisfy WCAG 2.1 Level AA ($\ge 4.5:1$ contrast ratio) against arbitrary dynamic background colors $(R, G, B)$ with relative luminance $L = 0.2126 \cdot R_{\text{linear}} + 0.7152 \cdot G_{\text{linear}} + 0.0722 \cdot B_{\text{linear}}$:

1. **Contrast against pure white foreground ($L_w = 1.0$):**
   $$\text{Contrast}_{\text{white}} = \frac{1.0 + 0.05}{L + 0.05} = \frac{1.05}{L + 0.05} \ge 4.5 \iff 1.05 \ge 4.5(L + 0.05) \iff L \le \frac{1.05}{4.5} - 0.05 \approx 0.1833$$

2. **Contrast against near-black foreground ($L_b \approx 0.0$):**
   $$\text{Contrast}_{\text{dark}} = \frac{L + 0.05}{0.0 + 0.05} = 20 L + 1.0 \ge 4.5 \iff 20 L \ge 3.5 \iff L \ge 0.175$$

The standards-compliant crossover threshold is established at $L_{\text{threshold}} = 0.179$:
$$\text{BackgroundStyle}(L) = \begin{cases} \text{MacBackgroundStyle::Dark} \implies \text{On-Accent White Text } (L_w = 1.0) & \text{if } L \le 0.179 \\ \text{MacBackgroundStyle::Light} \implies \text{Dark / Near-Black Text } (L_b \approx 0.0) & \text{if } L > 0.179 \end{cases}$$

This guarantees $\text{Contrast Ratio} \ge 4.5:1$ across all dynamic accent fills and container surfaces.

---

## 5. Strategy 2: Configurable Window Controls (`WindowControlsStyle`)

Window caption controls SHALL adhere to **Strategy 2 (Token-Driven / Configurable Presentation)**, allowing runtime or compile-time switching between native macOS aesthetics, Windows Fluent aesthetics, or OS-managed decorations.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    #[default]
    AppleTrafficLights,
    WindowsCaptionButtons,
    NativeHost,
}
```

### 5.1 `AppleTrafficLights` (Leading / Left-Edge macOS Style)
- **Placement & Margins:**
  - Anchored to the leading (left) edge with $18.0\,\text{pt}$ margin (`toolbar.traffic_lights_inset` = 70.0pt total clearance).
  - Centered vertically within the unified toolbar height.
- **Discs Geometry:**
  - Radius $r = 6.0\,\text{pt}$ (diameter $12.0\,\text{pt}$, token `toolbar.traffic_light_radius`).
  - Spacing pitch $= 20.0\,\text{pt}$ ($8.0\,\text{pt}$ inter-button gap, token `toolbar.traffic_light_spacing`).
  - Total cluster width: $52.0\,\text{pt}$.
- **Semantic Colors:**
  - Close: Fill `traffic_light_close` (`#FF5F56`), stroke $0.5\,\text{pt}$ `traffic_light_close_border` (`#E0443E`).
  - Minimize: Fill `traffic_light_minimize` (`#FFBD2E`), stroke $0.5\,\text{pt}$ `traffic_light_minimize_border` (`#DEA123`).
  - Zoom: Fill `traffic_light_zoom` (`#27C93F`), stroke $0.5\,\text{pt}$ `traffic_light_zoom_border` (`#1AAB29`).
- **Cluster-Wide Hover Affordance:**
  - Hovering ANY portion of the traffic lights cluster reveals glyphs on ALL three discs simultaneously.
  - Close: Diagonal cross `✕`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Minimize: Horizontal bar `—`, span $5.6\,\text{pt}$, stroke $1.1\,\text{pt}$.
  - Zoom: Plus sign `+`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 5.2 `WindowsCaptionButtons` (Trailing / Right-Edge Fluent Style)
- **Placement & Fitts's Law Conformance:**
  - Anchored to the trailing (right) edge with $0.0\,\text{pt}$ margin: the rightmost edge of the Close button touches the window's physical edge ($x = W_{\text{window}}$), strictly satisfying Fitts's law for corner flings.
  - Sequence: Minimize (`—`), Maximize/Restore (`□` / `❐`), Close (`✕`).
- **Dimensions & Sizing:**
  - Button width: $46.0\,\text{pt}$ (`toolbar.caption_button_width`).
  - Button height: Spans full titlebar height budget.
  - Total group width: $138.0\,\text{pt}$.
- **Hover & Active States:**
  - Minimize / Maximize: Neutral hover fill (`rgba(255, 255, 255, 0.08)` dark, `rgba(0, 0, 0, 0.06)` light).
  - Close Button: Crimson red hover fill `#E81123`. While hovered, glyph stroke transitions to solid white (`#FFFFFF`).
- **Vector Glyphs:**
  - Minimize: Horizontal stroke `—`, span $10.0\,\text{pt}$, $1.0\,\text{pt}$ stroke width.
  - Maximize: Square outline `□`, $10.0 \times 10.0\,\text{pt}$, $1.0\,\text{pt}$ stroke width. Maximize-restore renders dual overlapping $8.0\,\text{pt}$ squares.
  - Close: $45^\circ$ diagonal cross `✕`, $10.0 \times 10.0\,\text{pt}$, $1.0\,\text{pt}$ stroke width.

### 5.3 `NativeHost` Delegation
- Disables client-side synthesized window controls completely.
- Window decorations (title, minimize, maximize, close) are provided natively by the operating system window manager.
- Reclaims toolbar margins for custom client items.
