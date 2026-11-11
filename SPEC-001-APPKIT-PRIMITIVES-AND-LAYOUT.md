# SPEC-001: Deterministic AppKit Layout Engine & Consumer Parity Specification

- **Document ID:** SPEC-001
- **Status:** Approved / Normative
- **Version:** 0.6.8
- **Scope:** Standalone crate `egui-macview` and its integration into consumer applications on Windows 11, macOS, and Linux.
- **Normative Standards:** ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG Specification, ITU-T/IEEE SSIM, SemVer 2.0.0.

---

## 1. Normative Industry Standards & Decomposition

This document defines the overarching specification framework for `egui-macview`. The formal modular sub-specifications are organized in the `specs/` directory:

1. [**SPEC-001**](specs/SPEC-001-DESIGN-TOKENS-AND-WCAG.md): Design Tokens, Optical Tracking & WCAG AA Contrast.
2. [**SPEC-002**](specs/SPEC-002-CONTROLS-AND-MICRO-ANIMATIONS.md): Authentic AppKit Controls & Micro-Animation Oracles.
3. [**SPEC-003**](specs/SPEC-003-LAYOUT-ENGINES.md): Deterministic Layout Engines & Pixel-Accurate macOS Window Framing.
4. [**SPEC-004**](specs/SPEC-004-CORE-ANIMATION-PHYSICS.md): CoreAnimation Spring Physics Runtime (`CASpring`).
5. [**SPEC-005**](specs/SPEC-005-WINDOW-FRAMING-AND-CONTROLS.md): Window Framing, Drag Handling & Configurable Caption Controls.
6. [**SPEC-006**](specs/SPEC-006-INSPECTOR-ARCHITECTURE.md): Three-Tier Inspector Architecture & Floating Surfaces.

---

## 2. System Architecture & MVC Data Model

The presentation architecture is decoupled into three distinct responsibilities:

```
+------------------------------------------------------------------------+
|                            MODEL (State)                              |
|  - MacControlState: enabled, hovered, pressed, focused, window_state   |
|  - LocalEditBuffer: active text buffer, dirty flag, validation status  |
|  - WindowControlsStyle: AppleTrafficLights, WindowsCaptionButtons,      |
|                         NativeHost                                     |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                         CONTROLLER (Layout)                            |
|  - MacFixedView / MacRow / MacStackView: AppKit layout engines         |
|  - TabFocusChain: deterministic focus navigation across input fields   |
|  - Window drag handling & corner-fling hit areas (Fitts's Law)         |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                          VIEW (Rendering)                              |
|  - Background: 1px border stroke, subpixel antialiasing                |
|  - Content: SF Pro / Inter typography with optical tracking tables     |
|  - Caption Controls: Apple traffic lights (left) or Windows (right)    |
|  - Overlay: Window-level focus ring (NSFocusRingPlacementOutside)       |
+------------------------------------------------------------------------+
```

---

## 3. Configurable Window Controls (`WindowControlsStyle`)

Window caption controls follow a token-driven presentation model, dynamically switching between macOS traffic lights, Windows 11 caption buttons, or OS-managed decorations:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    #[default]
    AppleTrafficLights,
    WindowsCaptionButtons,
    NativeHost,
}
```

### 3.1 `AppleTrafficLights` (Leading / Left-Anchored macOS Style)
- **Placement & Margins:**
  - Anchored on the leading left edge with $18.0\,\text{pt}$ inset (`toolbar.traffic_lights_inset` = 70.0pt total margin including cluster).
  - Vertically centered inside the unified toolbar ($38.0\,\text{pt}$ / $52.0\,\text{pt}$).
- **Geometry Metrics:**
  - Radius $r = 6.0\,\text{pt}$ (diameter $12.0\,\text{pt}$, token `toolbar.traffic_light_radius`).
  - Spacing $= 8.0\,\text{pt}$ (pitch $20.0\,\text{pt}$, token `toolbar.traffic_light_spacing`).
  - Total cluster width: $52.0\,\text{pt}$.
- **Semantic Colors:**
  - Close: Fill `traffic_light_close` (`#FF5F56`), stroke $0.5\,\text{pt}$ `traffic_light_close_border` (`#E0443E`).
  - Minimize: Fill `traffic_light_minimize` (`#FFBD2E`), stroke $0.5\,\text{pt}$ `traffic_light_minimize_border` (`#DEA123`).
  - Zoom: Fill `traffic_light_zoom` (`#27C93F`), stroke $0.5\,\text{pt}$ `traffic_light_zoom_border` (`#1AAB29`).
- **Cluster-Wide Hover Affordance:**
  - Hovering over any part of the cluster reveals symbols simultaneously on all three buttons.
  - Close: Diagonal cross `✕`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Minimize: Horizontal bar `—`, span $5.6\,\text{pt}$, stroke $1.1\,\text{pt}$.
  - Zoom: Plus sign `+`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 3.2 `WindowsCaptionButtons` (Trailing / Right-Anchored Fluent Style)
- **Placement & Fitts's Law:**
  - Anchored on the trailing right edge with $0.0\,\text{pt}$ margin: the outer edge of the Close button touches the window edge ($x = W_{\text{window}}$) supporting corner clicks per Fitts's law.
  - Order: Minimize (`—`), Maximize/Restore (`□` / `❐`), Close (`✕`).
- **Dimensions:**
  - Button width: $46.0\,\text{pt}$ (`toolbar.caption_button_width`).
  - Button height: Spans the full height of the title bar.
  - Total group width: $138.0\,\text{pt}$.
- **Hover & Active States:**
  - Minimize / Maximize: Neutral hover background (`rgba(255, 255, 255, 0.08)` dark, `rgba(0, 0, 0, 0.06)` light).
  - Close button: Crimson hover background `#E81123`. On hover, cross glyph renders crisp white (`#FFFFFF`).
- **Vector Glyphs:**
  - Minimize: Horizontal line `—`, span $10.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Maximize: Square outline `□`, $10.0 \times 10.0\,\text{pt}$, stroke $1.0\,\text{pt}$. Restore mode displays two overlapping $8.0\,\text{pt}$ squares.
  - Close: $45^\circ$ diagonal cross `✕`, $10.0 \times 10.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 3.3 `NativeHost`
- Disables synthetic window buttons entirely.
- The host operating system provides native decorations.

---

## 4. Container Surface Awareness & AppKit Vibrancy Engine (`v0.6.8`)

Child controls placed inside or geometrically over grouped container surfaces (`MacBox`, `MacCard`, or custom `MacTheme::with_surface` scopes) MUST automatically detect their underlying surface color and resolve vibrant, WCAG 2.1 Level AA compliant foreground typography tints with zero per-frame heap allocations.

### 4.1 Surface-Relative Vibrancy (`PlusDarker` / `PlusLighter`) & WCAG 2.1 AA Clamping
- **Light Surfaces (`MacBackgroundStyle::Light`, $L > 0.179$):** Foreground label tokens (`label`, `secondary_label`, `tertiary_label`) are composited via `plus_darker` (`kCGBlendModePlusDarker`), shifting channels darker in proportion to the per-channel delta $|C_{\text{window}} - C_{\text{surface}}| \cdot g_{\text{vibrancy}}$ (`label.vibrancy_gain = 1.5`).
- **Dark Surfaces (`MacBackgroundStyle::Dark`, $L \le 0.179$):** Foreground label tokens are composited via `plus_lighter` (`kCGBlendModePlusLighter`), shifting channels lighter in proportion to $|C_{\text{window}} - C_{\text{surface}}| \cdot g_{\text{vibrancy}}$.
- **WCAG 2.1 AA Contrast Clamping (`ensure_wcag_contrast`):** A 10-step allocation-free binary search toward the high-contrast pole (`#000000` on light surfaces, `#FFFFFF` on dark surfaces) enforces:
  - $\text{Contrast Ratio} \ge 4.5:1$ (`label.wcag_aa_normal_ratio`) for `MacTheme::label_on` and `MacTheme::secondary_label_on`.
  - $\text{Contrast Ratio} \ge 3.0:1$ (`label.wcag_aa_large_ratio`) for `MacTheme::tertiary_label_on`.

### 4.2 Dual-Mode Zero-Allocation Surface Resolution (Closure Scopes & Flat NIB Geometry)
AppKit layouts construct grouped sections in two ways: (a) hierarchical closure nesting (`MacBox::show(ui, |ui| { ... })`, `MacCard::show(ui, rect, |ui, _| { ... })`) and (b) flat Interface Builder (NIB) coordinate layouts where `MacBox::paint_at(ui, box_rect)` or `MacBox::fixed_rect(box_rect).show(ui, |_| {})` paints the background bezel first and peer controls (`MacLabel`, `MacCheckbox`, `MacDisclosureGroup`, `MacInlineActionButton`) are placed afterward outside the closure using `.fixed_rect(control_rect)`.

To support both idioms deterministically without heap allocation:
1. **Native `egui::Ui` Hierarchy Propagation (`child_ui.visuals_mut().panel_fill`):** `MacBox::show` and `MacCard::show` set `child_ui.visuals_mut().panel_fill = surface_bg` on their scoped child `Ui`, automatically propagating the container surface to all nested children and automatically restoring the parent `panel_fill` when `child_ui` drops.
2. **Per-Pass Geometric Surface Registry (`PASS_SURFACE_RECTS`):** `MacBox::paint_at`, `MacBox::show`, and `MacCard::show` register `(rect, surface_bg)` in painter z-order into a fixed-capacity `Copy` thread-local array scoped to the active `egui::Context` and `ctx.cumulative_pass_nr()`.
3. **Deterministic Precedence (`MacTheme::active_surface_at(ui, pos)`):**
   1. Last registered `(rect, surface_bg)` in `PASS_SURFACE_RECTS` for the current pass where `rect.contains(pos)` (topmost finalized container wins for flat NIB `.fixed_rect` siblings).
   2. Custom `ui.visuals().panel_fill` (propagated from an enclosing `MacBox::show` / `MacCard::show` `child_ui` or explicitly set by the caller).
   3. Explicit `MacTheme::with_surface` scope override.
   4. Fallback to `MacTheme::window_background()`.
4. **Shared NIB Baseline & Visual Bezel Clamping (`MacTheme::baseline_text_y`, `MacTheme::clamp_nib_visual_rect`):** All NIB text and bezel controls (`MacLabel`, `MacInputCell`, `MacPopUpButton`, `MacComboBox`, `MacCheckbox`, `MacHelpButton`, `MacInlineActionButton`) share unified `MacTheme::baseline_text_y` (`firstBaselineOffsetFromTop - ascender`) and `MacTheme::clamp_nib_visual_rect` helpers and honor `ui.is_enabled()` (`ui.add_enabled_ui(false, ...)`).

### 4.3 Normative Architecture Guard
- `tests/architecture_guard_tests.rs` (`test_guard_surface_aware_vibrancy_anchoring`) statically verifies that `src/controls/box_view.rs` and `src/surfaces/card.rs` propagate surfaces via `visuals_mut().panel_fill` and `register_surface_rect_slot`, that all label/caption controls resolve foreground colors through `active_surface_at` / `active_surface` and `*_label_on`, and that NIB controls call `baseline_text_y` and `clamp_nib_visual_rect` instead of duplicating baseline or clamping math inline.

