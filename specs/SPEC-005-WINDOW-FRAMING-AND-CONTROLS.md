# SPEC-005: Window Framing, Drag Handling & Configurable Caption Controls

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-005 |
| **Document Title** | Window Framing, Drag Handling & Configurable Caption Controls |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/micro_animations_and_hud_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, Microsoft Fluent Guidelines, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: Apple Traffic Lights & Window Dragging |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Strategy 2 Configurable WindowControlsStyle (AppleTrafficLights, WindowsCaptionButtons, NativeHost) & Fitts's Law Conformance |

---

## 1. Scope & Strategy 2 Presentation

Window caption controls SHALL adhere to **Strategy 2 (Token-Driven / Configurable Presentation)**, allowing runtime or compile-time switching between native macOS aesthetics, Windows Fluent aesthetics, or OS-managed decorations.

---

## 2. Window Framing & Drag Invariants

- **Host Viewport Configuration:** Windows using custom synthesized titlebars MUST configure `with_decorations(false)` and `with_transparent(true)`.
- **Unified Toolbar / Titlebar:** Single integrated top area spanning the full window width, height resolved from `toolbar.height` ($38.0\,\text{pt}$ or $52.0\,\text{pt}$).
- **Interactive Dragging:** Non-control toolbar surface space invokes `ui.ctx().send_viewport_cmd(ViewportCommand::StartWindowDrag)`. Double-clicking toolbar whitespace toggles window maximization.

---

## 3. `WindowControlsStyle` Specification

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    #[default]
    AppleTrafficLights,
    WindowsCaptionButtons,
    NativeHost,
}
```

### 3.1 `AppleTrafficLights` (Leading / Left-Edge macOS Style)
- **Placement & Margin:**
  - Anchored to the leading (left) edge with $18.0\,\text{pt}$ margin (`toolbar.traffic_lights_inset`).
  - Centered vertically within the toolbar height.
- **Discs Geometry:**
  - Radius $r = 6.0\,\text{pt}$ (diameter $12.0\,\text{pt}$, token `toolbar.traffic_light_radius`).
  - Pitch $= 20.0\,\text{pt}$ ($8.0\,\text{pt}$ inter-button gap, token `toolbar.traffic_light_spacing`).
  - Total cluster width: $52.0\,\text{pt}$.
- **Semantic Coloration:**
  - Close: Fill `traffic_light_close` (`#FF5F56`), stroke $0.5\,\text{pt}$ `traffic_light_close_border` (`#E0443E`).
  - Minimize: Fill `traffic_light_minimize` (`#FFBD2E`), stroke $0.5\,\text{pt}`traffic_light_minimize_border` (`#DEA123`).
  - Zoom: Fill `traffic_light_zoom` (`#27C93F`), stroke $0.5\,\text{pt}`traffic_light_zoom_border` (`#1AAB29`).
- **Cluster-Wide Hover Affordance:**
  - Hovering ANY portion of the traffic lights cluster reveals glyphs on ALL three discs simultaneously.
  - Close: Diagonal cross `✕`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Minimize: Horizontal bar `—`, span $5.6\,\text{pt}$, stroke $1.1\,\text{pt}$.
  - Zoom: Plus sign `+`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 3.2 `WindowsCaptionButtons` (Trailing / Right-Edge Fluent Style)
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

### 3.3 `NativeHost` Delegation
- Disables client-side synthesized window controls completely.
- Window decorations (title, minimize, maximize, close) are provided natively by the operating system window manager.
- Reclaims toolbar margins for custom client items.
