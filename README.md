# `egui-macview`

**A Deterministic macOS AppKit Layout Engine, Authentic Control Primitive Suite, and CoreAnimation Physics Runtime for egui.**

---

## Overview

`egui-macview` brings native macOS AppKit precision, mathematical layout determinism, and CoreAnimation physics to the `egui` ecosystem.

While standard `egui` relies on content-driven streaming flow (where widgets automatically calculate margins and push the cursor), `egui-macview` introduces formal NIB-style retained layout bounds and explicit control framing.

For the normative specification suite, mathematical coordinate transformations, MVC datamodel, and iterative implementation roadmap, see the [`egui-macview Specification Suite`](specs/SPEC-INDEX.md).

---

## Core Capabilities

1. **Deterministic Layout Engines (`MacFixedView`, `MacFlexView`, `MacRow`, `MacSplitView`, `MacScrollView`, `MacStackView`)**:
   - Zero-margin framing guaranteeing mathematically proven zero-clipping layouts.
   - Exact token-driven compact inputs (`metrics.control.text_field.compact_infobox.height`), tool buttons (`metrics.toolbar.hit_target_min`), and hairline dividers (`metrics.split_view.divider_thickness`).
   - `MacSplitView`: Authentic AppKit `NSSplitView` divider with hairline rendering, drag interactions, and min/max constraints (`metrics.split_view.*`).
   - `MacRow`: Auto-budgeting horizontal cell layout with analytical $O(n)$ space distribution.
   - `MacScrollView`: Floating overlay scrollbars with zero-gutter reservation and automatic fade timer (`metrics.layout.scroll_view.*`).
   - `MacStackView`: Auto-layout linear stacking container with token-governed spacing and section separators (`metrics.layout.stack_view.*`).
2. **Authentic macOS Controls**:
   - `MacInputCell`: Compact input (`NSTextField` equivalent) with FSM cyclic tab-chain, external focus ring, and transactional buffer commit on enter/blur.
   - `MacButtonCell`: Native template image tinting using Apple accent and secondary colors (`NSButton`).
   - `MacSegmentedControl`: Apple segmented pill with animated slide indicators (`NSSegmentedControl`).
   - `MacToggle`: Native macOS toggle switch with token-driven dimensions and system green fill (`NSSwitch`).
   - `MacSlider`: Native continuous tracking slider with tick marks and token-driven metrics (`NSSlider`).
   - `MacThumbnailCell`: Generic preview canvas cell (`metrics.control.thumbnail.size`) with consumer-provided rendering callback (`NSImageCell`).
   - `MacOutlineView`: Hierarchical tree with disclosure chevrons, indentation steps, and customizable accessory slots (`NSOutlineView`).
   - `MacPopUpButton`: Trailing double chevrons with dropdown selection overlay (`NSPopUpButton`).
   - `MacSearchField`: Leading vector magnifying glass, trailing clear button, and focus retention (`NSSearchField`).
   - `MacStepper`: Discrete up/down scalar incrementing with float epsilon wrapping protection (`NSStepper`).
   - `MacColorWell`: Color swatch preview with translucent alpha checkerboard underlay (`NSColorWell`).
3. **CoreAnimation Physics (`CASpring`)**:
   - Closed-form analytical solution of the damped harmonic oscillator.
   - Exact support for Apple's perceptual spring model (`response` and `dampingRatio` tokens from `macview.motion.spring.default`).
   - Frame-rate independent: identical micro-second motion across 60Hz, 120Hz ProMotion, and 144Hz displays.

---

## Interactive Showcase Gallery

To run the interactive desktop gallery demonstrating the complete macOS AppKit suite on Windows 11:

```bash
cargo run --example macview_gallery
```

### Gallery Features:
- **Dynamic Theme Toggle**: Real-time switching between Aqua (Light) and DarkAqua (Dark) appearances.
- **MacSplitView Layout**: Dual hairline draggable splitters dividing Left Sidebar, Center Canvas, and Right Inspector with min/max constraint protection.
- **Left Sidebar (MacOutlineView)**: Hierarchical outline tree with disclosure chevrons, `MacThumbnailCell` vector drawings, and `accessory_custom` toggle buttons.
- **Center Canvas (MacScrollView & CASpring)**: `MacSearchField` filter bar, floating overlay scrollbars with zero gutter inflation, `MacFixedView` exact point positioning, `MacRow` auto-budgeted layout, and `MacPopover` powered by analytical damped harmonic oscillator physics.
- **Right Inspector (MacStackView)**: Collapsible sections demonstrating `MacInputCell` (cyclic Tab-chain FSM and focus rings), `MacSlider` (continuous tracking with tick marks), `MacStepper`, `MacPopUpButton`, `MacSegmentedControl`, `MacToggle`, and `MacColorWell` with alpha checkerboard underlays.

---

## Getting Started

```rust
use egui_macview::{
    theme::{MacAppearance, MacTheme},
    MacButtonCell, MacFixedView, MacInputCell, MacRow,
};
use egui::{pos2, vec2, Rect};

fn render_my_mac_panel(ui: &mut egui::Ui, theme: &MacTheme) {
    let input_height = theme.text_field().regular.height.points();
    let panel_rect = Rect::from_min_size(pos2(100.0, 100.0), vec2(280.0, input_height + 16.0));
    let fixed = MacFixedView::new(ui, panel_rect);

    let mut value = String::from("45.0");
    let input_rect = fixed.subview_rect(pos2(8.0, 8.0), vec2(80.0, input_height));

    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(input_rect));
    MacInputCell::new("input_cell", &mut value)
        .theme(theme)
        .width(80.0)
        .show(&mut child_ui);
}
```

