# `egui-macview`

**A Deterministic macOS AppKit Layout Engine, Authentic Control Primitive Suite, and CoreAnimation Physics Runtime for egui.**

---

## Overview

`egui-macview` brings native macOS AppKit precision, mathematical layout determinism, and CoreAnimation physics to the `egui` ecosystem.

While standard `egui` relies on content-driven streaming flow (where widgets automatically calculate margins and push the cursor), `egui-macview` introduces formal NIB-style retained layout bounds and explicit control framing.

For detailed cross-platform rendering mechanics, coordinate transformations, and hardware comparison between macOS, Windows, and Linux, see [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Core Capabilities

1. **Deterministic Layout (`MacFixedView` & `MacFlexView`)**:
   - Zero-margin framing guaranteeing mathematically proven zero-clipping layouts.
   - Exact $18.0\,\text{pt}$ compact inputs, $24.0\,\text{pt}$ tool buttons, and $1.0\,\text{pt}$ hairlines.
2. **Authentic macOS Controls**:
   - `MacInputCell`: Compact input with 1px border and native baseline alignment.
   - `MacButtonCell`: Native template image tinting using Apple accent and secondary colors.
   - `MacSegmentedControl`: Apple segmented pill with animated slide indicators.
   - `MacToggle`: Native macOS toggle switch.
3. **CoreAnimation Physics (`CASpring`)**:
   - Closed-form analytical solution of the damped harmonic oscillator.
   - Exact support for Apple's perceptual spring model (`response` and `dampingRatio`).
   - Frame-rate independent: identical micro-second motion across 60Hz, 120Hz ProMotion, and 144Hz displays.

---

## Getting Started

```rust
use egui_macview::layout::MacFixedView;
use egui_macview::controls::MacInputCell;
use egui::{pos2, vec2, Rect};

fn render_my_mac_panel(ui: &mut egui::Ui) {
    let panel_rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(250.0, 63.0));
    let mut view = MacFixedView::new(ui, panel_rect);

    let mut lsb = 45.0;
    view.place_control(
        Rect::from_min_size(pos2(4.0, 24.0), vec2(54.0, 18.0)),
        |ui, rect| {
            MacInputCell::new(&mut lsb).ui(ui, rect);
        },
    );
}
```
