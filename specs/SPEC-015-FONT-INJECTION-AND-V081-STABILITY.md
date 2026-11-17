# SPEC-015: External Font Injection API, Pro-Inspector Micro-Styling & v0.8.1 Stability Suite

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-015 |
| **Document Title** | External Font Injection API, Pro-Inspector Micro-Styling & v0.8.1 Stability Suite |
| **Document Version** | `0.8.1` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.8.1 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, W3C DTCG, WCAG 2.1 AA |

---

## 1. Scope & Architectural Purpose

`SPEC-015` defines the normative requirements for Milestone `0.8.1` (`egui-macview` stability & legal hygiene release):
1. **100% Legally Watertight Open-Source Typography & External Host Font Injection API (`setup_custom_macview_fonts`)**.
2. **Application-Agnostic Pro-Inspector Micro-Styling** (semantic green `+` and red `-` hover feedback, `1.0pt` input/stepper micro-bevels, and focused row highlight bands).
3. **Interactive Stability Fixes** across `MacContextThumbDial`, `MacColorWell`, `MacSymbols`, `MacPlayerView`, `MacCollectionView`, and `macview_gallery`.

*(Note: Next-generation optical glass material rendering is deferred to Milestone `0.9.0` pending per-widget architectural specification.)*

---

## 2. Normative Requirements

### REQ-015-01: Legally Watertight Open-Source Typography & External Font Injection API
1. `egui-macview` SHALL bundle and reference only MIT / SIL OFL open-source fonts (`Inter-Regular`, `Inter-SemiBold`, and `CupertinoIcons`) in all UI labels, gallery demonstrations, and runtime defaults. Hardcoded UI labels referencing proprietary third-party font trademarks (such as `"SF Pro"`) in gallery controls or runtime widgets are strictly forbidden.
2. `egui-macview` SHALL expose [`setup_custom_macview_fonts`] accepting custom Regular and Semibold TrueType/OpenType byte slices (`regular_ttf: Vec<u8>`, `semibold_ttf: Option<Vec<u8>>`), installing them as the primary proportional and semibold typefaces while preserving `CupertinoIcons` fallback and `Hack` Monospace integrity. This enables host applications to inject custom licensed fonts externally without `egui-macview` embedding or naming proprietary font assets.

### REQ-015-02: Application-Agnostic Pro-Inspector Micro-Styling
1. **Input Cell & Caret Micro-Shadow**: `MacInputCell`, `MacSearchField`, `MacPopUpButton`, and `MacStepper` SHALL render a `1.0pt` micro-bevel/shadow (recessed top inner shadow on text inputs; subtle `1.0pt` bottom drop shadow on stepper/popup controls).
2. **Borderless Inspector Action Rollover**: Borderless inline action buttons SHALL remain flat at rest and render a subtle translucent grey rollover pill (`MacTheme::inspector_hover_fill`) when hovered.
3. **Semantic Add (`+`) Green & Remove (`-`) Red Hover Feedback**:
   - In [`MacInlineActionButton`], [`MacRuleEditor`], and [`MacTableActionBar`], hovering an **Add (`+`)** button SHALL tint the icon and subtle rollover pill with `MacTheme::system_green()`.
   - Hovering a **Remove (`-`)** button SHALL tint the icon and subtle rollover pill with `MacTheme::system_red()`.
4. **Focused Inspector Row Highlight (`focused_row_fill`)**: When any child control inside a `MacFormRow` holds keyboard focus (`Response::has_focus()`), the row container SHALL paint a subtle full-width horizontal background band (`MacTheme::focused_row_fill()`).

### REQ-015-03: `MacContextThumbDial` True Circularity, Pure Cupertino Icons, Spatial-Intent Click/Slide Ergonomics & Transient HUD Tooltip
1. **True Circular Outer Ring & Concave-Safe Annular Quad-Strip Cushions**:
   - Full `360°` rings in `MacContextThumbDial` SHALL be rendered via true circular primitives (`Painter::circle` / `Painter::circle_stroke`) with radial detent ticks.
   - Radial cushions (`N`, `NE`, `E`, `SE`, `S`, `W`) and the `Undo`/`Redo` crown crescents SHALL be tessellated as a concave-safe quad-strip `egui::Mesh` between `r_inner` and `r_outer` (never `Shape::convex_polygon`, which produces internal diagonal wireframe artifacts and clips concave inner arcs) with a closed perimeter stroke (`stroke_annular_sector`).
2. **6 Distinct Vector Icons per Preset & Cupertino History Icons (Zero Text Inside Small Cushions)**:
   - The 6 radial cushions and the `Undo`/`Redo` crescents SHALL NOT render inline text strings inside the small buttons.
   - Every tool preset (`Pen`, `Select`, `Primitives`, `Knife`) SHALL render 6 visually distinct vector icons across all 6 sectors (`N`, `NE`, `E`, `SE`, `S`, `W`), using composite badges for diagonal combinations (`NE`, `SE`) so no two sectors in the same preset share an identical icon.
   - `Undo` and `Redo` crescents SHALL render `MacSymbol::ArrowCounterClockwise` (`\u{f21c}`) and `MacSymbol::ArrowClockwise` (`\u{f49a}`) from `CupertinoIcons`.
3. **Spatial-Intent Laptop & Tablet Ergonomics (1-Click Sector Latch, Center-Slide Momentary, 1-Click Joystick Toggle)**:
   - **Direct Cushion Click/Tap = Toggle Latch**: Clicking or tapping directly on any of the 6 cushions (`< 600 ms` press duration, including single-frame clicks) without dragging out of the sector SHALL always toggle-latch that sector in 1 click.
   - **Center-to-Cushion Thumb Slide = Momentary Modifier**: Pressing inside the center hub (`⊙`) and sliding the thumb outward into a cushion (`started_in_hub && active_sector.is_some()`), or holding a cushion `>= 600 ms`, SHALL activate that sector momentarily while held and automatically spring back to neutral upon release.
   - **Direct Center Hub Click/Tap = Immediate 1-Click Layer Toggle (`CompassModifiers` $\leftrightarrow$ `FreeJoystickBowl`)**: Clicking or tapping the center hub (`⊙`) without sliding out to a sector SHALL always switch between the Pie Menu (`CompassModifiers`) and the 360° Joystick (`FreeJoystickBowl`) in **1 single click**, even if a sector was previously latched.
4. **Transient HUD Tooltip Pill (`1.0s` Hold + `0.35s` Fade-Out)**:
   - Hovering or touching any cushion or `Undo`/`Redo` crescent SHALL trigger a compact floating HUD tooltip pill positioned above the outer dial ring displaying the concise action name.
   - The tooltip SHALL remain at full opacity for `1.0s` after sector entry/activation and smoothly fade out over `0.35s` (`alpha: 1.0 -> 0.0`).

### REQ-015-04: True Monospace Font Family & Immediate Font Repaint
1. `setup_symbols_font` (`src/symbols/mod.rs`) SHALL NOT insert proportional `Inter-Regular` at index `0` of `FontFamily::Monospace` or `FontFamily::Name("Ubuntu-Light")`, preserving `egui`'s built-in fixed-width `Hack` monospace font and `Ubuntu-Light` typeface.
2. Changing the active typeface in Demo 2 (`Symbols & Layout Lab`) SHALL immediately invoke `ctx.request_repaint()`.

### REQ-015-05: `MacColorWell` Saturation Unlock, Full Popover Hit-Rect & Live Accent Propagation
1. When opening a `MacColorWell` popover on an achromatic color (`Saturation < 0.08`, such as `#141416` or `#ECECEC`), `MacColorWell` SHALL persist an `Hsva` state in `egui` temporary memory and automatically unlock saturation (`s = 0.85`, `v >= 0.88` when dark) via [`MacColorWell::unlock_achromatic_hsva`] when the user selects a hue.
2. Outside-click dismissal of the `MacColorWell` popover SHALL test against `area_resp.response.rect` so clicking the bottom Hue bar never closes the popover.
3. Selecting an Accent Color in Demo 3 (`accentIndex`) SHALL propagate to all accent-bound surfaces (`wcag_bg`, `symbol_tint`, and `thumb_glass_color`).

### REQ-015-06: Browser Local Disk Permission Picker (`wasm32`) & NIB Button Sizing
1. On `wasm32`, clicking `Toggle Local Disk` or `Choose...` in Demo 4 (`Finder`) SHALL trigger a browser file permission dialog and populate the Finder table with the selected local files.
2. Action buttons (`Open URL / Path`, `Toggle Local Disk`, `Copy`) in `macOS NIB Window & SwiftUI Studio` SHALL be sized with sufficient horizontal width (`>= 136pt`) to prevent label truncation.

### REQ-015-07: AVKit Media Speed Binding, Uninterrupted Browser Playback, Trim Clamping & VU Ballistics
1. The `Speed` rotary slider in Demo 5 (`AVKit Media Studio`) SHALL be bidirectionally bound to `media_speed`.
2. Active media playback (`media_playing == true`) SHALL advance time and request repaints regardless of pointer inactivity so browser playback never halts after 3 seconds.
3. The top `MacPlayerView` SHALL clamp playback progression to the active `[media_in_point, media_out_point]` trim range.
4. Audio meters SHALL apply smooth release ballistics (`12.0 dB/s` decay, `1.2s` peak hold) and throttle numerical dB readout labels (`vuTextL`, `vuTextR`) to `200 ms` (`5 Hz`) intervals to eliminate text flicker.

### REQ-015-08: Scrollable `MacCollectionView`, Exclusive Selection, Unified Compact Mode & Vector Artboard Demo
1. `MacCollectionView` inside `render_tab_view_node` SHALL be wrapped in a vertical `MacScrollView` with adequate viewport height and enforce exclusive single-item selection on click.
2. Toolbar and Inspector mode labels SHALL both read `Default | Compact | Pro`. Selecting `Compact` SHALL collapse both sidebars and dock the full-scale (`1.05x`) thumb wheel and context dial on the left canvas edge.
3. Demo 1 (`Vector & Inspector Studio`) SHALL feature a self-contained generic **Vector Artboard** (a geometric polygon with a blue selection bounding box, 8 white square resize handles, and a movable reference origin `⊙`) controlled live by all 6 `MacContextThumbDial` cushions, the `MacEdgeThumbwheel`, the `MacFreeJoystickBowl`, and the `Undo`/`Redo` stack.
4. **Zero Overlay Collision in Demo 1**: In Paired mode, `MacEdgeThumbwheel` SHALL sit cleanly above the `Undo`/`Redo` crown (`r_skirt_out + 6.0pt` clearance) without overlapping the `Undo`/`Redo` crescents, the top-left `HUD Palette`, or the top-right segmented mode strips.

### REQ-015-09: Crate Version `0.8.1` & Cache-Busted WASM Bundle
1. `egui_macview::VERSION` (`CARGO_PKG_VERSION`) SHALL equal `"0.8.1"`.
2. `examples/gallery/mobile_web_html.html` SHALL fetch `macview_gallery.wasm?v=0801` to prevent stale browser caching across desktop/mobile deployments.
