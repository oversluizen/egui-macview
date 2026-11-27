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
2. `egui-macview` SHALL expose [`setup_custom_macview_fonts`] accepting custom Regular and Semibold TrueType/OpenType byte slices (`regular_ttf: Vec<u8>`, `semibold_ttf: Option<Vec<u8>>`), and [`setup_custom_macview_font_suite`] accepting all 4 host font slots (`regular_ttf: Vec<u8>`, `semibold_ttf: Option<Vec<u8>>`, `light_ttf: Option<Vec<u8>>`, `monospace_ttf: Option<Vec<u8>>`), installing them as the primary proportional (`FontFamily::Proportional`), semibold (`FontFamily::Name("Inter-SemiBold")`), light (`FontFamily::Name("Ubuntu-Light")` / `MacTheme::light_font_id`), and monospace (`FontFamily::Monospace`) typefaces while preserving `CupertinoIcons` fallback. This enables host applications to inject custom licensed fonts externally without `egui-macview` embedding or naming proprietary font assets.

### REQ-015-02: Application-Agnostic Pro-Inspector Micro-Styling & Glyphs 4 Action Button Fidelity
1. **Input Cell & Caret Micro-Shadow**: `MacInputCell`, `MacSearchField`, `MacPopUpButton`, and `MacStepper` SHALL render a `1.0pt` micro-bevel/shadow (recessed top inner shadow on text inputs; subtle `1.0pt` bottom drop shadow on stepper/popup controls).
2. **Borderless Inspector Action Rollover (`GSMouseOverButton`)**: Borderless inline action buttons (including `MacDisclosureGroup` trailing `+`) SHALL remain completely borderless at rest and render a subtle `16×16pt` (`R = 3.5pt`) translucent rollover square (`MacTheme::inspector_hover_fill`) when hovered.
3. **Semantic Add (`+`) Green & Remove (`-`) Red Hover Feedback & Delicate Crosshair Proportions**:
   - In [`MacInlineActionButton`], [`MacRuleEditor`], [`MacDisclosureGroup`], and [`MacTableActionBar`], hovering an **Add (`+`)** button SHALL tint the icon and subtle rollover pill with `MacTheme::system_green()`.
   - Hovering a **Remove (`-`)** button SHALL tint the icon and subtle rollover pill with `MacTheme::system_red()`.
   - Circular stepper badges in [`MacRuleEditor`] (`16×16pt`) and [`MacInlineActionButton`] (`14×14pt`) SHALL draw a `1.0pt` circular stroke with delicate `3.2pt` half-arm crosshairs (`1.25pt` stroke) leaving ample interior breathing room, while [`MacTableActionBar`] SHALL render a flat Glyphs 4 Layers footer bar (`+  −  |  ⚙▾`) with `.with_action_menu(true)`.
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

### REQ-015-07: AVKit Media Speed Binding, Interactive Floating HUD, Unmuted Audio & VU Ballistics
1. The `Speed` rotary slider in Demo 5 (`AVKit Media Studio`) SHALL be bidirectionally bound to `media_speed`.
2. Active media playback (`media_playing == true`) SHALL advance time and request repaints regardless of pointer inactivity so browser playback never halts after 3 seconds.
3. `render_procedural_video_viewport` (`examples/gallery/video_scene.rs`) SHALL embed an interactive [`MacPlayerView`] (`MacPlayerControlsStyle::Floating`) inside the video viewport wired to `media_playing`, `media_sound_output` (`muted = !media_sound_output`), `media_gain`, `media_speed`, and `media_playhead` clamped to `[media_in_point, media_out_point]`, using `MacSymbol::Play` and `MacSymbol::Pause` (zero missing-glyph tofu `❚❚`).
4. `CenterState::default()` SHALL initialize `media_sound_output = true` and seed `audio_track` from the initial `.xib` option of `s5-pop-codec`, and `state5_avkit_media.rs` SHALL invoke `NativeAudioOutput::play_wav` whenever `media_playing && media_sound_output && !NativeAudioOutput::is_playing()`.
5. Audio meters SHALL apply smooth release ballistics (`12.0 dB/s` decay, `1.2s` peak hold), throttle numerical dB readout labels (`vuTextL`, `vuTextR`) to `200 ms` (`5 Hz`) intervals, and dynamically apply `s5-sw-vudisc` (`Discrete` vs `Continuous`) and `s5-sw-vuthr` (`Thresholds`) via `MacSceneDataSource::resolve_level_discrete` and `resolve_level_thresholds`.

### REQ-015-08: Scrollable `MacCollectionView`, Exclusive Selection, Unified Compact Mode & Vector Artboard Demo
1. `MacCollectionView` inside `render_tab_view_node` SHALL be wrapped in a vertical `MacScrollView` with adequate viewport height and enforce exclusive single-item selection on click.
2. Toolbar and Inspector mode labels SHALL both read `Default | Compact | Pro`. Selecting `Compact` SHALL collapse both sidebars and dock the full-scale (`1.05x`) thumb wheel and context dial on the left canvas edge.
3. Demo 1 (`Vector & Inspector Studio`) SHALL feature a self-contained generic **Vector Artboard** controlled live by all 6 `MacContextThumbDial` cushions, the `MacEdgeThumbwheel`, the `MacFreeJoystickBowl`, and the `Undo`/`Redo` stack, dynamically adapting its active vector visualization and manipulation to the selected tool preset (`active_thumb_preset`):
   - `Pen (P)`: cubic Bézier path with on-curve anchor nodes and off-curve control handles reacting live to `45 deg` angle snap, `Cusp` (sharp corner handle break), `G2` curvature continuity, and `Move`.
   - `Select (V)`: multi-node vector contour with selectable anchor nodes reacting live to `Axis` lock, `Clone` (ghost duplicate contour offset), `1-Side` handle constraint, and `Slide`.
   - `Primitives (F)`: geometric polygon with blue selection bounding box, 8 white square resize handles, and movable 9-point reference origin `⊙` reacting to `1:1` aspect lock, `Origin`, `Shear`, and `Radius`.
   - `Knife (E)`: live parametric slice line across the vector shape with entry/exit intersection markers reacting to `15 deg` angle snap, `Nodes` split preview, and `Snap`.
4. **Zero Overlay Collision in Demo 1**: In Paired mode, `MacEdgeThumbwheel` SHALL sit cleanly above the `Undo`/`Redo` crown (`r_skirt_out + 6.0pt` clearance) without overlapping the `Undo`/`Redo` crescents, the top-left `HUD Palette`, or the top-right segmented mode strips.

### REQ-015-09: Crate Version `0.8.1` & Cache-Busted WASM Bundle
1. `egui_macview::VERSION` (`CARGO_PKG_VERSION`) SHALL equal `"0.8.1"`.
2. `examples/gallery/mobile_web_html.html` SHALL fetch `macview_gallery.wasm?v=0801` to prevent stale browser caching across desktop/mobile deployments.

### REQ-015-10: Deterministic Artboard Transform Controller, $15^\circ$ Detent Stepping & Stationary Card Center
1. **Stationary Card Center on Origin Anchor Change**: Changing `origin_anchor` (`MacOriginAnchor`) in Demo 1 (`Vector & Inspector Studio`) SHALL move **exclusively** the reference origin crosshair (`⊙`, `origin_local`) across the 9 anchor points of the card's bounding box (`TopLeft` .. `BottomRight`) without adding any positional offset `(ox, oy)` to `card_center`. The artboard card itself SHALL remain stationary when `origin_anchor` changes.
2. **Detent-Gated Discrete Origin Stepping & Rotary Taptic Events**:
   - In `MacContextThumbDial`, dragging on the outer rotary ring SHALL accumulate angular progress and emit `MacTapticEvent::RotaryDetentTick` **only** when crossing a discrete $15^\circ$ (`detent_step_deg`) detent boundary, never on sub-degree pointer jitter (`0.01°`).
   - In Demo 1 (`state1_thumb_overlay.rs`), stepping `origin_anchor` via the `Origin` (`MacThumbIntent::Alter`) cushion SHALL accumulate angular delta (`origin_detent_accum_deg`) and advance `origin_anchor` **only** once per full $15^\circ$ detent step (`|accum| >= 15.0°`), never on every render frame (`> 1e-4°`).
3. **Decoupled Edge Thumbwheel & Sector-Switch Flywheel Brake**:
   - `MacEdgeThumbwheel` in Demo 1 SHALL bind to a dedicated `thumbwheel_angle_deg` state variable decoupled from `card.rotation_deg` and consume `w_resp.value_delta`.
   - Switching the active/latched sector on `MacContextThumbDial` SHALL immediately reset `origin_detent_accum_deg` and halt any residual `MacEdgeThumbwheel` flywheel inertia (`MacEdgeThumbwheel::brake_flywheel(ctx, id)`), preventing flywheel carry-over into newly selected sectors.

### REQ-015-11: Battery-Efficient Timed Repaints, Single-Pass NIB Sync & NASA/IBM Cleanroom Modularity
1. **Timed Repaints (`request_repaint_after`)**:
   - `MacContextThumbDial` SHALL NOT invoke `ui.ctx().request_repaint()` on every frame during the static `1.0s` tooltip hold window; instead it SHALL schedule a timed wake (`request_repaint_after_secs`) until the fade window begins, and pace fade frames at `16 ms` (`60 Hz`).
   - `MacEdgeThumbwheel` free-spin flywheel coasting SHALL schedule `request_repaint_after(Duration::from_millis(16))` (`60 Hz` ceiling) instead of unthrottled `request_repaint()`.
2. **Single-Pass Toast Layout & Zero-Clone Active Screen NIB Rendering**:
   - `MacToastManager::show` SHALL compute each toast's text galley **once** per toast per frame (`layout_no_wrap`), reusing the same `Arc<Galley>` for both pill width calculation and painting.
   - `CenterState::render_active_screen_nib` SHALL render the active `MacSceneIR` via `Arc<MacSceneIR>` (zero deep `MacSceneIR::clone()` per frame), and `sync_all_nib_stores` SHALL synchronize only the **active** screen's store (`active_screen`), the active right inspector store, and (when open) the `nib_studio` store once per frame phase.
3. **Shared Binary `NIBArchive` Graph & Shared Liquid Glass Ring/Arc Primitives**:
   - `decode_real_nib_archive_v1` (`src/scene/nib_varint.rs`) SHALL reuse `IbNibGraph::parse` (`src/scene/nib_ibtool.rs`) as the single source of truth for `NIBArchive\x01\x00\x00\x00` header and table parsing.
   - `MacFreeJoystickBowl` and `MacContextThumbDial` SHALL share unified dock-rect allocation, curved `Undo`/`Redo` crescent hit-testing and painting (`MacSymbol::ArrowCounterClockwise` / `MacSymbol::ArrowClockwise`), and quadratic joystick physics helpers.

### REQ-015-12: Zero Literal-Arithmetic Token Bypass & 100% Data-Driven NIB/Token Resolution
1. **Explicit W3C DTCG Tokens for `RuleEditorMetrics` & `InlineActionButtonMetrics` (Zero Literal-Chain Arithmetic)**:
   - `tokens/design_tokens.json` and `src/tokens/` SHALL define `badge_diameter` (`16.0pt`), `glyph_half_span` (`3.2pt`), and `glyph_stroke_width` (`1.25pt`) on [`RuleEditorMetrics`], and `glyph_half_span` (`3.2pt`) on [`InlineActionButtonMetrics`].
   - Chained literal arithmetic (`1.0 + 0.5`, `- 2.0 - 2.0`, `* 0.5 - 0.5`) to synthesize un-tokenized dimensions in `src/controls/` is strictly forbidden and enforced by `check_token_arithmetic_cheating`.
2. **100% Data-Driven `.xib` Help Popovers, Foreground Layering, Options, Media Titles & Scene Purity (`REQ-001-06`, `REQ-013-07`)**:
   - Every `<button bezelStyle="helpButton">` across all 7 showcase `.xib` assets SHALL carry a non-empty `toolTip` attribute preserved across `XibXmlFrontend`, `scene_to_xcode16_xib_xml`, `ibtool --compile`, and `NibArchiveFrontend` (`NSIBHelpConnector` `NSMarker` / `NSToolTip`), with zero non-standard `ibTip`, `ibTitle`, `ibSize`, `ibIcon`, `ibDesignFrame`, or `ibId` `userDefinedRuntimeAttribute` crutches.
   - `SceneNodeKind::HelpButton` in `src/scene/runtime_widgets.rs` SHALL render its anchored [`MacPopover`] inside an `egui::Order::Foreground` `egui::Area` with wrapped text (`TextWrapMode::Wrap`) so it floats above all sibling `MacBox` containers and dismisses on outside click or `Escape`.
   - `src/scene/`, `src/motion/`, and `examples/gallery/` SHALL contain zero hardcoded domain/gallery tabs or criteria (`runtime_tabs.rs` renders child `SceneNode`s / `render_tab_item` of `<tabViewItem>`; `RuleEditor` reads criteria from `MacSceneDataSource`; `LevelIndicator` and `Stepper` read thresholds/bounds from `SceneNode`), zero duplicate `.xib` option arrays or hardcoded Y-coordinate enablement ranges in `examples/gallery/`, zero hand-painted fake segmented/color-well pills in `state1_thumb_overlay.rs`, zero `ui.add_space(...)` calls, zero magic numbers in `src/scene/` and `src/motion/`, and zero `Color32::from_rgb` / `Color32::WHITE` / `Color32::BLACK` calls in `src/` or `examples/gallery/`.

### REQ-015-13: `MacCanvasFooter` Hit-Rect Integrity, Cross-Screen Zoom Synchronization & Dynamic OS/Size Profiles
1. **`MacCanvasFooter` Zero-Overflow Zoom Cluster**:
   - `MacCanvasFooter::show` (`src/navigation/footer.rs`) SHALL set `zoom_ui.spacing_mut().item_spacing.x = 0.0` before allocating the `50% | 100% | 200%` preset buttons with explicit `zoom_gap` spacing so all 3 buttons lie 100% within `zoom_area` and respond reliably to clicks.
2. **Cross-Screen `50% | 100% | 200%` Zoom Synchronization**:
   - Selecting a zoom preset (`0.5`, `1.0`, `2.0`) in `MacCanvasFooter` SHALL immediately synchronize `card.scale_pct`, the active `.nib` data stores, and the active screen's zoom/scale state (`card.scale_pct` on Screen 1, `symbol_preview_size_idx` and `typo_point_size` on Screen 2, `showcase_control_size_idx` on Screen 3, `media_zoom` on Screen 5, and `nib_check_*` on Screen 6).
3. **Dynamic OS Profile & Control Size Resolution in State 6 & Paired `NumberField` Sync**:
   - Selecting an OS profile in `s6-en-pop-preset` (`nibPresetIdx`) or toggling `Regular (14pt)` (`s6-en-chk-reg`), `Small (12pt)` (`s6-en-chk-sml`), or `Mini Mixed (10pt)` (`s6-en-chk-mni`) in State 6 SHALL dynamically update the rendered [`MacControlSize`] of the `s6-en-*` controls via `MacSceneDataSource::resolve_control_size` as well as the active unit, width, and codec selection.
   - `ShowcaseSceneDataStore::seed_from_scene` and `pull_store_live_from_gallery` SHALL keep paired `Slider` and `NumberField` bindings (`mediaGain`, `scalePercent`, `rotationDeg`, `cornerRadius`) synchronized without stale `"100"` defaults, formatting fractional values (`|v| <= 10.0`) with two decimals.



