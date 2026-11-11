# SPEC-001: Deterministic AppKit Layout Engine & Consumer Parity Specification

- **Document ID:** SPEC-001
- **Status:** Approved / Normative
- **Version:** 0.6.7
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
