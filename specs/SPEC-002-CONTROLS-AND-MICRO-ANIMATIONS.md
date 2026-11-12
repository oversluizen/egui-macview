# SPEC-002B: Authentic AppKit Controls & Micro-Animation Oracles

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-002B |
| **Document Title** | Authentic AppKit Controls & Micro-Animation Oracles |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/appkit_primitives_tests.rs`, `tests/micro_animations_and_hud_tests.rs`, `tests/high_assurance_invariants_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: MacButtonCell, MacInputCell, MacSegmentedControl, MacToggle |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Full Control Suite (MacColorWell, MacOutlineView, MacPopUpButton, MacSearchField, MacSlider, MacStepper, MacThumbnailCell) & Micro-Animation Oracles |
| `0.4.1` | 2026-11-08 | Architecture Team | Consolidation Hardening: Disambiguated Document ID (`SPEC-002B`), transactional `MacInputCell` buffer, `CASpring` micro-animations, and foreground click isolation |
| `0.6.11` | 2026-11-12 | Architecture Team | AppKit `NSSegmentedControl` Parity: `SegmentItem` (`Text`, `Symbol`, `Custom` template closure), per-segment `width`/`enabled`/`tooltip`, orthogonal `has_menu` & `shows_menu_indicator`, press-and-hold / pull-down / secondary menu triggers, and Sonoma active-pill divider suppression |

---

## 1. Scope & Normative Foundation

This specification defines the complete suite of authentic macOS AppKit control primitives in `egui-macview`. Every control is mathematically bounded, resolves dimensions from design tokens, and features physics-based micro-animations.

---

## 2. Control Primitives

### 2.1 `MacInputCell` (`NSTextField` Equivalent)
- **Dimensions & Geometry:** Standard height resolved from token `text_field.compact_infobox.height` ($18.0\,\text{pt}$) or `text_field.regular.height` ($22.0\,\text{pt}$), corner radius from `text_field.*.corner_radius` ($4.0\,\text{pt}$), and border stroke of $1.0\,\text{pt}$ (`separator` color token).
- **Baseline Alignment:** Fixed typographic baseline at $14.0\,\text{pt}$ from the top edge of the cell, ensuring vertical alignment across adjacent inputs and labels.
- **FSM Tab-Chain Focus Traversal:**
  - Each cell accepts a unique identifier and an optional `next_focus_id`.
  - Pressing `Tab` transitions keyboard focus cyclically to the next field within the same rendering pass.
  - Pressing `Shift+Tab` navigates in reverse order.
- **Transactional Commit Model:**
  - Text input is buffered in a local string during interactive editing.
  - Changes are committed to the consumer data model exclusively upon `Enter`, `Tab`, or focus loss (`blur`).
  - Pressing `Escape` rolls back changes to the initial value and relinquishes focus.
- **Explicit ID Constructor:** `MacInputCell::with_id(id, value)` prevents double-hashing and automatically requests focus on cell click.

### 2.2 `MacButtonCell` (`NSButton` Equivalent)
- **Styling:** Standard push button, toolbar icon button, and accessory button variants with geometry derived from `button.regular.height` ($22.0\,\text{pt}$) and `button.regular.corner_radius` ($6.0\,\text{pt}$).
- **Template Image Tinting:** Monochrome template images are dynamically tinted using system accent (`control_accent` token) or secondary content color (`placeholder_text` token).
- **States:** Distinct visual feedback for normal, hovered, pressed, disabled, and window-inactive states.

### 2.3 `MacSegmentedControl` (`NSSegmentedControl` Equivalent)
- **Geometry & Distribution (`setWidth(_:forSegment:)`):** Height resolved from token `metrics.controls.segmented.{regular,small,mini}.height` ($22.0 / 18.0 / 15.0\,\text{pt}$, overridable via `.height(...)` or `.fixed_rect(...)`), container corner radius from `segmented.*.corner_radius`, and optional explicit per-segment point widths via `SegmentItem::with_width`. Unspecified segments distribute across remaining width or size to content plus horizontal padding (`segmented.padding_horizontal`).
- **Segment Content (`SegmentItem`):** Supports text labels (`SegmentItem::new`), embedded vector symbols (`SegmentItem::from_symbol`), and consumer-supplied monochrome template vector closures (`SegmentItem::from_icon(|painter, rect, tint| ...)`), receiving interior rect inset by `segmented.icon_padding` and surface-aware WCAG-compliant AppKit state tint (`label_on`, `secondary_label_on`, or disabled `tertiary_label_on`).
- **Per-Segment State (`setEnabled(_:forSegment:)`, `setToolTip(_:forSegment:)`):** Individual segments MAY be disabled via `SegmentItem::enabled(false)`, rendering in disabled tint, ignoring pointer input, and being skipped during `ArrowLeft` / `ArrowRight` keyboard focus traversal.
- **Active Pill & Sonoma Divider Suppression:** Animated sliding pill behind the selected segment using `CASpring` motion physics (`segmented.indicator_inset`, `segmented.*.indicator_radius`). In rounded style, interior vertical dividers (inset by `segmented.divider_vertical_inset`) render exclusively between two inactive adjacent segments and are suppressed on both edges of the active pill.
- **Orthogonal Segment Menus (`setMenu(_:forSegment:)` & `setShowsMenuIndicator(_:forSegment:)`):**
  - `SegmentItem::with_menu(true)` enables menu activation via: (a) press-and-hold ($\ge$ `segmented.menu_press_hold_delay`, $0.25\,\text{s}$), (b) downward drag pull ($\ge$ `segmented.menu_drag_threshold`, $4.0\,\text{pt}$), (c) secondary click / `Control`+click, or (d) keyboard `ArrowDown` / `Shift+F10` while focused (`SegmentMenuTrigger::Keyboard`), emitting `SegmentEvent { index, menu_trigger: Some(...), rect }` while quick primary clicks select the segment (`menu_trigger: None`). Segments with `has_menu == false` never emit menu triggers.
  - `SegmentItem::with_shows_menu_indicator(true)` renders a crisp two-segment analytical downward vector chevron (`⌄`) anchored by DTCG tokens (`menu_chevron_half_width`, `menu_chevron_height`, `menu_chevron_stroke_width`, `menu_chevron_inset_trailing`, `menu_chevron_inset_bottom`) with zero polygon approximation or token arithmetic.
- **Pressed Highlight (`NSSegmentedCell.isHighlighted`) & Momentary Tracking (`NSSegmentedSwitchTrackingMomentary`):** Holding the primary pointer button over an enabled inactive segment renders a token-driven depressed backdrop (`segmented.pressed_highlight_opacity`) and primary foreground tint while the pointer remains within that segment's bounds, cancelling visually and behaviorally if dragged outside before release. In momentary mode (`MacSegmentedControl::momentary`, `.show_momentary`, `.show_momentary_with_events`), no persistent active pill is drawn at rest; segments highlight exclusively while depressed, fire their action upon release (or `Space` / `Enter` on the keyboard cursor), and preserve full segment menu trigger parity (`PressAndHold`, `PullDown`, `SecondaryClick`, `Keyboard`).
- **Accessibility (`NSAccessibilityRadioGroup` / WCAG 4.1.2), Keyboard Parity & Deterministic ID Salt:** Single-selection controls register `egui::Role::RadioGroup` (`WidgetInfo`) with optional `.accessibility_label(...)` and active segment value; `show_multi` and `show_momentary` register `egui::Role::Toolbar`, paint an AppKit focus ring when focused, traverse enabled segments via `ArrowLeft` / `ArrowRight`, and activate/toggle via `Space` / `Enter`. Optional `.id_salt(impl Hash + Debug)` anchors widget identity deterministically across conditional peer insertions.
- **Stack-Inline Hot Path:** Controls with up to `MacSegmentedControl::INLINE_SEGMENT_CAPACITY` ($8$) segments store descriptors and per-segment layout slots inline on the stack without per-frame `Vec` heap allocations, seamlessly spilling to heap storage above $8$ segments.

### 2.4 `MacToggle` (`NSSwitch` Equivalent)
- **Geometry:** Width resolved from token `metrics.control.toggle.width` ($26.0\,\text{pt}$), height from `metrics.control.toggle.height` ($15.0\,\text{pt}$), and fully rounded pill corner radius from `metrics.control.toggle.corner_radius` ($7.5\,\text{pt}$).
- **Indicator Thumb:** Circular white knob with diameter from `metrics.control.toggle.knob_size` ($13.0\,\text{pt}$), smoothly translated via `CASpring` physics between disabled offset `metrics.control.toggle.knob_offset_off` ($1.0\,\text{pt}$) and enabled offset `metrics.control.toggle.knob_offset_on` ($12.0\,\text{pt}$).
- **Drop Shadow:** Knob renders an ambient drop shadow ($0.0\,\text{pt}$ horizontal, $1.0\,\text{pt}$ vertical offset, $2.0\,\text{pt}$ blur radius at $20\%$ opacity).
- **Coloration:** Background animates between neutral inactive fill (`color.{appearance}.control`) and system green accent fill (`color.{appearance}.system_green`).

### 2.5 `MacSlider` (`NSSlider` Equivalent)
- **Geometry:** Continuous or discrete linear track, track thickness resolved from token `metrics.control.slider.track_height` ($3.0\,\text{pt}$), thumb diameter from `metrics.control.slider.thumb_size` ($16.0\,\text{pt}$), thumb border stroke from `metrics.control.slider.thumb_stroke_width` ($1.0\,\text{pt}$).
- **Tracking:** Linear mapping between normalized interval $[0.0, 1.0]$ and consumer range $[v_{\min}, v_{\max}]$.
- **Drop Shadow:** Thumb renders an ambient drop shadow ($0.0\,\text{pt}$ horizontal, $1.5\,\text{pt}$ vertical offset, $2.5\,\text{pt}$ blur radius at $18\%$ opacity) over the track.
- **Tick Marks:** Optional rendering of discrete tick mark indicators along the track with dimensions from `metrics.control.slider.tick_height` ($4.0\,\text{pt}$) and `metrics.control.slider.tick_width` ($1.0\,\text{pt}$).
- **Keyboard Control:** Left/Down arrow decrements by step fraction from token `metrics.control.slider.key_step_fraction`, Right/Up arrow increments by step fraction.

### 2.6 `MacThumbnailCell` (`NSImageCell` Generic Canvas Preview)
- **Geometry:** Bounded display cell resolved from token `metrics.control.thumbnail.size` ($24.0 \times 24.0\,\text{pt}$), corner radius from `metrics.control.thumbnail.corner_radius` ($4.0\,\text{pt}$), border stroke from `metrics.control.thumbnail.border_width` ($1.0\,\text{pt}$).
- **Rendering Model:** Accepts a consumer-provided closure `FnOnce(&Painter, Rect)` to draw arbitrary vector or raster preview graphics.
- **Framing & Aspect Ratio:** Automatically applies subpixel snapping to cell boundaries and provides an isolated clipping or scaling rect.
- **Background & Border:** Optional perimeter border resolved from token `color.{appearance}.separator` and subtle background fill from `color.{appearance}.control_background`.

### 2.7 `MacOutlineView` (`NSOutlineView` Hierarchical Tree)
- **Hierarchy Representation:** Multi-level tree supporting arbitrary nesting depth with configurable indentation step resolved from token `metrics.control.outline_view.indent_step` ($16.0\,\text{pt}$ per level), row height from `metrics.control.outline_view.row_height` ($24.0\,\text{pt}$).
- **Disclosure Chevron:** Triangular disclosure indicator with span from `metrics.control.outline_view.chevron_size` ($7.0\,\text{pt}$) and stroke from `metrics.control.outline_view.chevron_stroke_width` ($1.2\,\text{pt}$) rotating $90^\circ$ smoothly between collapsed ($0^\circ$) and expanded ($90^\circ$) states.
- **Row Interaction:** Single-click selection, double-click action trigger, and hover highlighting (tokens `color.{appearance}.selected_control` and `color.{appearance}.control`).
- **Accessory Slots:** Trailing accessory slot (`accessory_custom`) with hit targets from `metrics.control.outline_view.accessory_size` ($16.0\,\text{pt}$) for action buttons.

### 2.8 `MacPopUpButton` (`NSPopUpButton` Equivalent)
- **Role:** Discrete selection dropdown button displaying the currently active item with a trailing stacked double-chevron indicator.
- **Geometry & Alignment:**
  - Intrinsic height resolved from token `metrics.control.popup_button.height` ($22.0\,\text{pt}$), squircle corner radius from `metrics.control.popup_button.corner_radius` ($5.0\,\text{pt}$), horizontal padding from `metrics.control.popup_button.padding_horizontal` ($8.0\,\text{pt}$).
  - Trailing indicator: Stacked double chevrons (up/down pair, span from `metrics.control.popup_button.chevron_size` ($8.0\,\text{pt}$), stroke from `metrics.control.popup_button.chevron_stroke_width` ($1.2\,\text{pt}$), spacing from `metrics.control.popup_button.chevron_spacing` ($6.0\,\text{pt}$)) rendered in `color.{appearance}.placeholder_text` tint.
- **State & Interaction:** Space/Enter opens dropdown; Up/Down traverses items.

### 2.9 `MacSearchField` (`NSSearchField` Equivalent)
- **Role:** Dedicated single-line text input for filtering and querying with integrated leading search icon and trailing cancel/clear button.
- **Geometry & Alignment:**
  - Intrinsic height resolved from token `metrics.control.search_field.height` ($22.0\,\text{pt}$), fully rounded pill corner radius from `metrics.control.search_field.corner_radius` ($11.0\,\text{pt}$).
  - Leading icon: Vector-rendered optical magnifying glass with span from `metrics.control.search_field.icon_size` ($12.0\,\text{pt}$), stroke from `metrics.control.search_field.icon_stroke_width` ($1.2\,\text{pt}$) rendered in `color.{appearance}.placeholder_text` tint.
  - Trailing clear button: Circular glyph with diameter from `metrics.control.search_field.clear_button_size` ($14.0\,\text{pt}$), displayed exclusively when `!text.is_empty()`.

### 2.10 `MacStepper` (`NSStepper` Equivalent)
- **Role:** Stacked dual-arrow vertical stepper button providing discrete scalar increment/decrement adjustments.
- **Geometry & Alignment:**
  - Compact width resolved from token `metrics.control.stepper.width` ($14.0\,\text{pt}$), height from `metrics.control.stepper.height` ($22.0\,\text{pt}$), corner radius from `metrics.control.stepper.corner_radius` ($4.0\,\text{pt}$).
  - Horizontal hairline divider with thickness from `metrics.control.stepper.divider_thickness` ($1.0\,\text{pt}$) bisecting the cell at $y = \text{height} / 2.0$.
  - Symmetrical top/bottom arrow chevrons with span from `metrics.control.stepper.arrow_size` ($6.0\,\text{pt}$) and stroke from `metrics.control.stepper.arrow_stroke_width` ($1.2\,\text{pt}$) pointing up ($\wedge$) and down ($\vee$).
- **Interaction & Bounds Clamping:** Top click increments by `step`, bottom click decrements, strictly clamped or wrapped.

### 2.11 `MacColorWell` (`NSColorWell` Equivalent)
- **Role:** Color swatch preview box displaying current RGBA color state with interaction affordance for palette selection.
- **Geometry & Alignment:**
  - Standard bezel width resolved from token `metrics.control.color_well.width` ($44.0\,\text{pt}$), height from `metrics.control.color_well.height` ($23.0\,\text{pt}$), corner radius from `metrics.control.color_well.corner_radius` ($5.0\,\text{pt}$), border stroke from `metrics.control.color_well.border_width` ($1.0\,\text{pt}$).
  - Swatch compartment: Inset from `metrics.control.color_well.swatch_inset` ($3.0\,\text{pt}$) with inner corner radius from `metrics.control.color_well.swatch_radius` ($3.0\,\text{pt}$).
  - Checkerboard underlay: Rendered grid pattern with square pitch from `metrics.control.color_well.checker_grid_size` ($4.0\,\text{pt}$) for translucent alpha preview when $\alpha < 255$.

---

## 3. Micro-Animation Oracles & Vector Rendering Rules

1. **Analytical Physics Transitions:** Interactive control state changes SHALL NOT switch discretely or instantaneously. All transitions MUST evaluate via analytical `CASpring` kinematics.
2. **Strict Vector Path Oracle:** Stepper arrows, popup chevrons, search icons, and window caption glyphs SHALL be rendered as analytical vector paths with subpixel snapping. Cheating via Unicode or ASCII characters (`^`, `v`, `+`, `x`) is strictly forbidden.
