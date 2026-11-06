# SPEC-006: Three-Tier Inspector Architecture & Floating Surfaces

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-006 |
| **Document Title** | Three-Tier Inspector Architecture & Floating Surfaces |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/cots_red_baseline_tests.rs`, `tests/micro_animations_and_hud_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: Compact HUD Palette & MacPanel Floating Surfaces |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Three-Tier Inspector Architecture (Compact HUD, Default NIB, Expanded Pro) & MacPopover Directional Arrows |

---

## 1. Scope & Inspector Typology

Professional desktop software requires adaptable property inspection matching workflow intensity. `egui-macview` defines three standardized inspector tiers and lightweight floating surfaces.

---

## 2. Three-Tier Inspector Presets

### 2.1 Mode 0: Default NIB Document Inspector
- **Role:** Standard right-hand property sidebar for desktop document applications (Finder, Pages, Keynote).
- **Geometry & Alignment Constraints:**
  - Panel Width: Fixed or clamped range from token `metrics.split_view.inspector_min` ($260.0\,\text{pt}$) to $320.0\,\text{pt}$.
  - Control Height: Standard AppKit control height $= 22.0\text{--}24.0\,\text{pt}$.
  - Section Spacing: $8.0\,\text{pt}$ padding, $1.0\,\text{pt}$ hairline dividers, $11.0\,\text{pt}$ bold/secondary section headers.

### 2.2 Mode 1: Compact Pro HUD Palette (`NSPanelStyleMaskHUDWindow`)
- **Role:** Floating semi-translucent dark property palette optimized for pro creative software (Glyphs, Final Cut Pro, Motion, Logic Pro).
- **Geometry & Alignment Constraints:**
  - Panel Dimensions: Standard compact width $= 178.0\,\text{pt}$, height $= 63.0\,\text{pt}$ (two-row dense layout).
  - Control Height: Compact input cell height $= 18.0\,\text{pt}$.
  - Insets & Padding: Horizontal cell padding $= 6.0\,\text{pt}$, inter-row vertical spacing $= 4.0\,\text{pt}$, section spacing $= 8.0\,\text{pt}$.
  - Surface Styling: Dark charcoal translucent fill `rgba(30, 30, 30, 0.88)` with $1.0\,\text{pt}$ hairline border `rgba(255, 255, 255, 0.12)` and $8.0\,\text{pt}$ panel corner radius.
  - Input Bezel: Recessed dark input background `rgba(20, 20, 20, 0.90)` with subtle $1.0\,\text{pt}$ border `rgba(255, 255, 255, 0.08)`.
  - Typography: $11.0\,\text{pt}$ system font for compact labels, $10.0\,\text{pt}$ secondary font for unit annotations.
- **Mathematical Zero-Clipping Guarantee:** Total required height $22.0\,\text{pt}$ (header) $+ 37.0\,\text{pt}$ (2 rows of 18pt $+ 1\text{pt}$ gap) $= 59.0\,\text{pt} \le 63.0\,\text{pt}$ panel budget $\implies$ $4.0\,\text{pt}$ surplus margin. Clipping is provably impossible.

### 2.3 Mode 2: Expanded Pro Inspector (`NSWindow` Multi-Deck Inspector)
- **Role:** Comprehensive multi-deck property inspector for deep parameter inspection (Xcode Inspector, Glyphs 4 Multi-Master Palette).
- **Geometry & Alignment Constraints:**
  - Panel Width: Expanded width $= 380.0\,\text{pt}$ (providing sufficient horizontal budget for multi-column matrix parameters and vector curves).
  - Tabbed Deck Navigation: Top-level segmented control tabs (`Attributes`, `Metrics`, `Advanced Matrix`) with animated `CASpring` pill transitions.
  - Deck Compartments:
    1. *Attributes Deck:* Core identity, layers, and interactive color well swatch.
    2. *Metrics Deck:* Bounding box, sidebearings, optical margin nudging with steppers and continuous sliders.
    3. *Advanced Matrix Deck:* Full 2x2 Affine transformation matrix $[a, b; c, d] + [t_x, t_y]$ with numerical precision stepping and live canvas updates.
  - Collapsible Sections: Native accordion decks with rotating vector chevrons and real-time canvas coordination.

---

## 3. Floating Surfaces: `MacPanel` & `MacPopover`

### 3.1 `MacPanel`
- **Role:** Floating HUD panel with dark translucent acrylic background, 1px border stroke, and drop shadow.
- **Dismissal & Occlusion:** Captures clicks to prevent background pass-through while active.

### 3.2 `MacPopover`
- **Role:** Transient anchor-relative callout popover with directional arrow callout indicator (`MacPopoverArrowEdge`: Top, Bottom, Left, Right).
- **Arrow Geometry:** Symmetrical triangular path with $10.0\,\text{pt}$ base width and $8.0\,\text{pt}$ altitude, seamlessly merged into the popover squircle border path.
- **Light Dismiss:** Dismisses automatically upon mouse click outside the popover bounding rect or upon pressing `Escape`.
