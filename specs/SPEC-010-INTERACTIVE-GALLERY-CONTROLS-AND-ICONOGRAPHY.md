# SPEC-010: Interactive Gallery Controls, Bidirectional State Binding & Iconography Parity

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-010 |
| **Document Title** | Interactive Gallery Controls, Bidirectional State Binding & Iconography Parity |
| **Document Version** | `0.3.1` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.3.1 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suites** | `tests/micro_animations_and_hud_tests.rs`, `tests/symbol_tests.rs`, `tests/appkit_primitives_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Jaco Oversluizen / `egui-macview` contributors | Initial Specification: Functional Binding and Cross-Platform Shortcut Parity |
| `0.3.1` | 2026-11-06 | Jaco Oversluizen / `egui-macview` contributors | Normative Release: Bidirectional Presets, Live Vector Transforms & SF Symbols |

---

## 1. Scope & System Architecture

### 1.1 Problem Statement & Architectural Justification
In desktop framework showcases, UI controls frequently degenerate into superficial visual mocks where sliders, steppers, and popup buttons render aesthetically but fail to alter underlying application state or execute promised actions ("dead UI"). Furthermore, macOS keyboard shortcuts (such as `⌘K`) fail on Windows platforms where no `Command` key exists, and pure text buttons reduce visual affordance and scanability.

To fulfill **Specification-Driven Verification** (Zero Mock Policy / High Engineering Integrity) and Apple Human Interface Guidelines (HIG):
1. **Zero Dead Controls:** Every interactive control presented in `macview_gallery` MUST be bidirectionally bound to live application state and trigger genuine real-time transformations.
2. **Cross-Platform Shortcut Parity:** Global shortcuts MUST adapt dynamically to the host operating system, binding `⌘K` on macOS and `Ctrl+K` (with fallback to `Win+K`) on Windows and Linux.
3. **Optically Scaled SF Symbols:** Pure text labels in segmented controls, action buttons, and navigation bars MUST be augmented with authentic vector glyphs from the embedded `CupertinoIcons` engine (`MacSymbol`), maintaining strict compliance with the crate's Anti-Emoji Guard.

---

## 2. Normative Requirements

### 2.1 Cross-Platform Keyboard Shortcut Parity
1. **Shortcut Ingestion:**
   The application SHALL intercept the global Spotlight Command Palette trigger across all supported host platforms:
   - On **macOS**: `egui::Modifiers::COMMAND + egui::Key::K`
   - On **Windows & Linux**: `egui::Modifiers::CTRL + egui::Key::K` and `egui::Modifiers::MAC_CMD + egui::Key::K`
2. **Event Consumption:**
   The shortcut SHALL be captured via `ui.input_mut(|i| i.consume_key(...))` to guarantee that the key press is consumed by the framework and does not bleed into focused text inputs.
3. **Dynamic Visual Labeling:**
   The toolbar search button label SHALL adapt at runtime to match the user's host platform:
   - On macOS: `format!("{} ⌘K", MacSymbol::Search.as_str())`
   - On Windows / Linux: `format!("{} Ctrl+K", MacSymbol::Search.as_str())`

### 2.2 Bidirectional Inspector Preset Synchronization
1. **Synchronization Invariant:**
   `self.inspector_mode` (`DefaultNib`, `CompactHud`, `ExpandedPro`) and the `Preset:` `MacPopUpButton` index in `render_default_nib` SHALL remain strictly synchronized:
   $$\text{selected\_index} = \begin{cases} 0 & \text{if } \text{mode} = \text{DefaultNib} \\ 1 & \text{if } \text{mode} = \text{CompactHud} \\ 2 & \text{if } \text{mode} = \text{ExpandedPro} \end{cases}$$
2. **Bidirectional Propagation:**
   - Modifying the preset via the `MacPopUpButton` MUST immediately transition `inspector_mode` to the selected target mode.
   - Modifying `inspector_mode` via the toolbar segmented control or sidebar navigation tree MUST immediately update the `MacPopUpButton` selected index.

### 2.3 Live Vector Transform Controls
1. **Continuous Scale Slider (`MacSlider`):**
   The scale slider in `nib_continuous_form` SHALL be bound directly to `CardModel.scale_pct` across the range `25.0%..=200.0%`. User drag operations MUST alter the affine 2D transform matrix in real time.
2. **Continuous Corner Radius Stepper (`MacStepper`):**
   The stepper in `nib_continuous_form` SHALL be bound directly to `CardModel.radius` across `0.0..=40.0 pt` with step size `1.0 pt`. Stepper increments MUST immediately update the card's visual corner radius and reflect the active measurement unit (`pt`, `px`, or `%`).
3. **Units Segmented Control (`MacSegmentedControl`):**
   Selecting `Points`, `Pixels`, or `Percent` in `nib_selection_form` SHALL dynamically re-format all inspector dimension readouts.
4. **Canvas Direct Touch / Click:**
   Mouse interaction on the vector canvas card SHALL trigger the `CASpring` harmonic oscillator bounce pulse:
   $$\text{scale}(t) = 1.0 + 0.20 \cdot |1.0 - p(t)| \cdot \sin(12.0 \cdot t)$$

### 2.4 Vector Iconography Integration (`MacSymbol`)
1. **Toolbar Inspector Mode Switcher:**
   Segmented items SHALL incorporate vector glyphs:
   - Default NIB: `MacSymbol::Gear` (`\u{f43c}`)
   - Compact HUD: `MacSymbol::Sliders` (`\u{f7dc}`)
   - Expanded Pro: `MacSymbol::Plus` (`\u{f489}`)
2. **Expanded Pro Multi-Deck Tabs:**
   Top segmented tab items SHALL incorporate vector glyphs:
   - Attributes: `MacSymbol::Sliders` (`\u{f7dc}`)
   - Metrics: `MacSymbol::Gear` (`\u{f43c}`)
   - Matrix: `MacSymbol::ArrowClockwise` (`\u{f49a}`)
3. **Action & Motion Buttons:**
   - CASpring Pulse: `MacSymbol::ArrowClockwise` (`\u{f49a}`)
   - Reset Matrix: `MacSymbol::ArrowCounterClockwise` (`\u{f21c}`)
4. **Anti-Emoji Guard Preservation:**
   Zero multi-colored host OS emoji characters are permitted in production or example source code. All iconography MUST resolve to monochrome PUA codepoints within `CupertinoIcons`.

---

## 3. Test Oracles & Verification Suite

| Test Identifier | Test Target | Verification Condition |
| :--- | :--- | :--- |
| `TC-1001-KBD-PARITY` | `consume_key` shortcut handler | Asserts both `Modifiers::COMMAND` and `Modifiers::CTRL` trigger palette toggle |
| `TC-1002-POPUP-SYNC` | `InspectorState::render_default_nib` | Asserts modifying `popup_selected` changes `inspector_mode` |
| `TC-1003-SLIDER-TRANSFORM` | `MacSlider` <-> `CardModel` | Asserts dragging slider updates `scale_pct` and card dimensions |
| `TC-1004-STEPPER-RADIUS` | `MacStepper` <-> `CardModel` | Asserts stepper actions round/unround card corner radius |
| `TC-1005-ANTI-EMOJI-GUARD` | `tests/symbol_tests.rs` | Verifies zero host emoji occurrences across the codebase |

---
*Normative specification accepted and verified under Automated Architecture Fitness Standards.*
