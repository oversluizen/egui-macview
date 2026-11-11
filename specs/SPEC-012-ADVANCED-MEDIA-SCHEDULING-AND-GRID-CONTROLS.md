# SPEC-012: Advanced Media, Scheduling, and Grid Controls

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-012 |
| **Document Title** | Advanced Media, Scheduling, and Grid Controls |
| **Document Version** | `0.6.8` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.6.8 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Hardware Ground Truth** | Apple Silicon MacBook Air (macOS Sonoma `14.8.9`, SDK `14`, 291 AppKit headers, 11 AVKit headers) |
| **Normative Test Suite** | `tests/spec012_advanced_controls_tests.rs`, `tests/cots_red_baseline_tests.rs`, `tests/architecture_guard_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, Apple AppKit / AVKit Reference, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Initial `egui-macview` specification suite baseline. |
| `0.6.0` | 2026-11-10 | Architecture Team | Milestone 0.6.0 Release: Advanced Media, Scheduling, and Grid Controls (`MacPlayerView`, `MacDatePicker`, `MacComboBox`, `MacTabView`, `MacDisclosureGroup`, `MacCollectionView`, `MacBox`, `MacRuleEditor`) grounded in live macOS Sonoma 14.8.9 hardware inspection, verified with `spec012_advanced_controls_tests.rs`. |
| `0.6.1` | 2026-11-10 | Architecture Team | Milestone 0.6.1 Release: Full AVKit transport/audio/speed controls, `MacRuleEditor` implementation, zero idle heap allocations (`MacTheme::resolve_or_default`), and `spec012_appkit_and_battery_parity_tests.rs`. |
| `0.6.2` | 2026-11-10 | Architecture Team | Milestone 0.6.2 Release: Comprehensive developer documentation, HIG decision tables, standalone `eframe` quickstart, IDE hover `/// # Example` doc-tests, and architecture diagrams. |
| `0.6.3` | 2026-11-10 | Architecture Team | Milestone 0.6.3 Release: Converted 62 `README.md` table links to rustdoc intra-doc links (`[`Type`]`) for `docs.rs`, added 9 `/// # Example` rustdoc blocks across Navigation, Surfaces, Feedback, State, and Symbols, and added `README.md` link guard. |
| `0.6.4` | 2026-11-10 | Architecture Team | Milestone 0.6.4 Release: Repository-wide 100% English comment/prose/package verification guard (`check_vault_leakage_and_dutch_in_text`), removal of legacy `AUDIT-REFACTORING-REPORT.md`, and portable `start_gallery.cmd`. |
| `0.6.5` | 2026-11-11 | Architecture Team | Milestone 0.6.5 Release: AppKit NIB parity (`MacCheckbox`, `MacLabel`, `MacHelpButton`, `MacInlineActionButton`), universal optical tracking (`layout_tracked_galley`), cross-platform baseline anchoring (`firstBaselineOffsetFromTop`), NIB `frame_rect` vs `visual_rect` bezel clamping, recessed `MacBox` surface tokens (`box_background`, `box_border`), `MacComboBox` right accent badge, `MacDisclosureGroup` trailing `+` action, and bundled open-source `Inter` font. |
| `0.6.6` | 2026-11-11 | Architecture Team | Milestone 0.6.6 Release: MacBook Air (`macOS Sonoma 14.8.9`) `NSTextFieldSquareBezel` corner-radius calibration (`2.0pt` Regular/Small/CompactInfobox, `1.5pt` Mini), WCAG 2.1 AA Aqua `secondary_label` contrast hardening (`#5A5A5F`, `5.48:1` on `box_background`), `MacHelpButton` control-size typography (`12/10/9pt` Semibold), `MacPopUpButton::accent_badge(bool)`, and zero-allocation `tracking_for_font_size` & `semibold_font_id` caching. |
| `0.6.7` | 2026-11-11 | Architecture Team | Milestone 0.6.7 Release: Dynamic AppKit Vibrancy (`plus_darker` / `plus_lighter`) & Surface-Aware WCAG 2.1 AA Contrast Engine (`ensure_wcag_contrast`, `MacTheme::label_on`, `secondary_label_on`, `tertiary_label_on`, `active_surface`, `with_surface`) automatically resolving `MacLabel`, `MacDisclosureGroup`, `MacCheckbox`, and `MacInlineActionButton` foreground colors on `MacBox` (`box_background`), `MacCard`, and custom container surfaces with zero per-frame heap allocations. |
| `0.6.8` | 2026-11-11 | Architecture Team | Milestone 0.6.8 Release: Zero-allocation per-pass geometric surface registry (`MacTheme::register_surface_rect`, `MacTheme::active_surface_at`) anchoring container surface awareness across both closure-scoped (`MacBox::show`, `MacCard::show`) and flat NIB `.fixed_rect` peer layouts (`MacBox::paint_at`, `MacBox::show`), guarded by `test_guard_surface_aware_vibrancy_anchoring` in `tests/architecture_guard_tests.rs`. |

---

## 1. Scope & Hardware-Calibrated Normative Foundation

This specification establishes the formal normative requirements, geometric models, micro-animation parameters, and test oracles for seven advanced AppKit and AVKit control suites slated for Milestone `0.6.0`:
1. **`MacPlayerView` (`AVPlayerView` Media Suite — `REQ-012-01`):** Complete AVKit media playback presentation layer supporting QuickTime floating HUD pod (`AVPlayerViewControlsStyleFloating`), inline bottom bar (`AVPlayerViewControlsStyleInline`), and minimal circular transport (`AVPlayerViewControlsStyleMinimal`).
2. **`MacDatePicker` (`NSDatePicker` — `REQ-012-02`):** Direct date and time scheduling control supporting text-field with integrated stepper (`NSDatePickerStyleTextFieldAndStepper`) across `Regular`, `Small`, and `Mini` sizes, plus visual clock and month calendar matrix grid (`NSDatePickerStyleClockAndCalendar`).
3. **`MacComboBox` (`NSComboBox` — `REQ-012-03`):** Hybrid single-line editable text input coupled with right dropdown trigger and viewport-clamped filterable popover list across `Regular`, `Small`, and `Mini` control sizes.
4. **`MacTabView` (`NSTabView` — `REQ-012-04`):** Multi-pane segmented container with flipped coordinate system (`isFlipped == 1`), bezel bordered tabs (`NSTopTabsBezelBorder`), and borderless card switching (`NSNoTabsNoBorder`).
5. **`MacDisclosureGroup` (`NSButton` Disclosure Primitives — `REQ-012-05`):** Hierarchical collapsible content toggle supporting classic triangular disclosure chevron (`NSBezelStyleDisclosure`) and rounded pill disclosure button (`NSBezelStyleRoundedDisclosure`) with `CASpring` rotation.
6. **`MacCollectionView` (`NSCollectionView` + `NSCollectionViewFlowLayout` — `REQ-012-06`):** High-performance 2D grid layout engine featuring 2D viewport culling (`ui.clip_rect()`), rubber-band marquee selection, and 2D spatial keyboard navigation.
7. **Supplementary AppKit Primitives (`REQ-012-07`):** Primary container grouping (`MacBox` / `NSBox`) and multi-clause filter/query builder (`MacRuleEditor` / `NSRuleEditor`).

All dimensional, visual, and behavioral constants in this specification are directly grounded in empirical extraction on macOS Sonoma `14.8.9` (Darwin 23.6.0, SDK 14, Apple M2 / Apple Silicon MacBook Air) via PyObjC introspection of private and public AppKit/AVKit headers.

---

## 2. W3C DTCG Token Definitions & Color Spaces

### 2.1 Media & Scheduling Design Tokens
All controls specified herein SHALL resolve styling exclusively via `MacTheme` backed by `tokens/design_tokens.json`. No magic pixel offsets, colors, or inline spring constants are permitted.

```json
{
  "player_view": {
    "floating_pod": {
      "width": { "$value": 481.0, "$type": "number" },
      "height": { "$value": 64.0, "$type": "number" },
      "corner_radius": { "$value": 16.0, "$type": "number" },
      "bottom_margin": { "$value": 16.0, "$type": "number" },
      "auto_hide_seconds": { "$value": 3.0, "$type": "number" }
    },
    "inline_bar": {
      "height": { "$value": 24.0, "$type": "number" },
      "corner_radius": { "$value": 0.0, "$type": "number" }
    },
    "scrubber": {
      "height": { "$value": 22.5, "$type": "number" },
      "track_height": { "$value": 3.5, "$type": "number" },
      "track_corner_radius": { "$value": 1.75, "$type": "number" },
      "knob_size": { "$value": 8.0, "$type": "number" },
      "trim_border_color": { "$value": "#FFD60A", "$type": "color" },
      "buffered_fill": { "$value": "rgba(255, 255, 255, 0.28)", "$type": "color" }
    }
  },
  "date_picker": {
    "stepper_width": { "$value": 19.0, "$type": "number" },
    "stepper_chevron_pitch": { "$value": 9.0, "$type": "number" },
    "selection_corner_radius": { "$value": 2.5, "$type": "number" },
    "calendar_width": { "$value": 139.0, "$type": "number" },
    "calendar_height": { "$value": 148.0, "$type": "number" },
    "clock_size": { "$value": 119.0, "$type": "number" }
  },
  "combo_box": {
    "intercell_spacing": { "x": 3.0, "y": 2.0 },
    "item_height": { "$value": 18.0, "$type": "number" },
    "visible_items": { "$value": 5, "$type": "number" }
  }
}
```

---

## 3. Normative Control Specifications & Mathematical Models

### 3.1 `MacPlayerView` (`AVPlayerView` Media Suite) — `REQ-012-01`
- **Reference:** [Apple Developer Documentation — `AVPlayerView`](https://developer.apple.com/documentation/avkit/avplayerview)
- **Control Styles:**
  - `AVPlayerViewControlsStyleFloating` (`2` — QuickTime HUD Pod):
    - **Pod Geometry:** Dimensions $481.0 \times 64.0\,\text{pt}$, corner radius $R = 16.0\,\text{pt}$, background material `NSVisualEffectMaterialHUDWindow` (`9`).
    - **Placement & Dismissal:** Positioned horizontally centered and $16.0\,\text{pt}$ above the bottom edge of the video viewport. In playback mode, automatically dims to $\alpha = 0.0$ after $3.0\,\text{s}$ of pointer inactivity; pointer motion within the video rect restores $\alpha = 1.0$ via `CASpring` transition.
    - **Top Row (Transport & Audio):** Height $34.0\,\text{pt}$.
      - Play/Pause Button: $29.0 \times 34.0\,\text{pt}$ (intrinsic symbol hitbox $29.0 \times 18.0\,\text{pt}$).
      - Skip Backward/Forward 15s: $25.0 \times 15.0\,\text{pt}$.
      - Frame Step Backward/Forward: $19.0 \times 21.0\,\text{pt}$.
      - Volume Button: $24.0 \times 17.0\,\text{pt}$, Volume Slider: $125.0 \times 18.0\,\text{pt}$.
      - AirPlay / Route Picker & Picture-in-Picture: $20.0 \times 20.0\,\text{pt}$.
      - Playback Speed Popover: Discrete speed menu $[0.5\times, 1.0\times, 1.25\times, 1.5\times, 2.0\times]$.
    - **Bottom Row (Timeline Scrubber):** Height $22.5\,\text{pt}$ (intrinsic $18.0\,\text{pt}$).
      - Track height $3.5\,\text{pt}$, track corner radius $R = 1.75\,\text{pt}$.
      - Buffered range bar filled with `rgba(255, 255, 255, 0.28)`.
      - Active In/Out trimming range bounded by `#FFD60A` yellow trim box.
      - Playhead needle knob: Circular $8.0 \times 8.0\,\text{pt}$ white disc with subtle drop shadow ($1.0\,\text{pt}$ blur, $\alpha = 0.25$).
      - Timecode Readouts: `.SFNS-Regular` $12.0\,\text{pt}$ monospaced digits. Elapsed time left ($34.0 \times 15.0\,\text{pt}$) formatted as `MM:SS`; remaining time right ($42.0 \times 15.0\,\text{pt}$) formatted as `-MM:SS`.
  - `AVPlayerViewControlsStyleInline` (`1`):
    - Fixed bottom control bar spanning $100\%$ of player width, fixed height $24.0\,\text{pt}$, corner radius $R = 0.0\,\text{pt}$.
  - `AVPlayerViewControlsStyleMinimal` (`3`):
    - Minimalist circular transport overlay: centered $29.0 \times 18.0\,\text{pt}$ Play/Pause toggle with concentric `AVCircularProgressIndicator` buffering ring.

### 3.2 `MacDatePicker` (`NSDatePicker`) — `REQ-012-02`
- **Reference:** [Apple Developer Documentation — `NSDatePicker`](https://developer.apple.com/documentation/appkit/nsdatepicker)
- **Control Sizes:** Supports `Regular`, `Small`, and `Mini`. (In native AppKit, `NSDatePicker` does not scale to `Large` / `NSControlSizeLarge`).
- **`NSDatePickerStyleTextFieldAndStepper` (`0`):**
  - Integrated right stepper: Invariant width $19.0\,\text{pt}$ across all sizes, vertical chevron pitch $9.0\,\text{pt}$.
  - Bezel corner radii: Regular $5.0\,\text{pt}$, Small $4.0\,\text{pt}$, Mini $3.0\,\text{pt}$.
  - Active segment highlight: Pill background with corner radius $R = 2.5\,\text{pt}$, tinted with `theme.selected_content_background()` (`#0064E1` Aqua / `#0059D1` DarkAqua).
  - **Date + Time (`NSDatePickerElementFlagYearMonthDay | NSDatePickerElementFlagHourMinuteSecond` = `0x00ec`):**
    - Regular: Frame $161 \times 28\,\text{pt}$ (intrinsic $158 \times 24\,\text{pt}$).
    - Small: Frame $157 \times 22\,\text{pt}$ (intrinsic $155 \times 20\,\text{pt}$).
    - Mini: Frame $155 \times 21\,\text{pt}$ (intrinsic $153 \times 20\,\text{pt}$).
  - **Date-Only (`NSDatePickerElementFlagYearMonthDay` = `0x00c0`):**
    - Regular: Frame $97 \times 28\,\text{pt}$ (intrinsic $94 \times 24\,\text{pt}$).
    - Small: Frame $93 \times 22\,\text{pt}$ (intrinsic $91 \times 20\,\text{pt}$).
    - Mini: Frame $91 \times 21\,\text{pt}$ (intrinsic $89 \times 20\,\text{pt}`).
  - **Time-Only (`NSDatePickerElementFlagHourMinuteSecond` = `0x000e`):**
    - Regular: Frame $81 \times 28\,\text{pt}$ (intrinsic $78 \times 24\,\text{pt}$).
    - Small: Frame $77 \times 22\,\text{pt}$ (intrinsic $75 \times 20\,\text{pt}`).
    - Mini: Frame $75 \times 21\,\text{pt}$ (intrinsic $73 \times 20\,\text{pt}$).
- **`NSDatePickerStyleClockAndCalendar` (`1`):**
  - **Date-Only (Month Calendar Grid):** Dimensions $139.0 \times 148.0\,\text{pt}$. Header navigation bar $24.0\,\text{pt}$, weekday label row $16.0\,\text{pt}$, $6 \times 7$ day matrix with $19.8 \times 18.0\,\text{pt}$ date cells.
  - **Time-Only (Analog Clock Face):** Dimensions $122.0 \times 123.0\,\text{pt}$ (intrinsic clock circle $119.0 \times 119.0\,\text{pt}$). Hour hand, minute hand, and center hub with smooth rotational kinematics.
  - **Combined (Calendar + Clock):** Dimensions $277.0 \times 148.0\,\text{pt}$ ($139.0\,\text{pt}$ calendar $+ 19.0\,\text{pt}$ gap $+ 119.0\,\text{pt}$ clock).

### 3.3 `MacComboBox` (`NSComboBox`) — `REQ-012-03`
- **Reference:** [Apple Developer Documentation — `NSComboBox`](https://developer.apple.com/documentation/appkit/nscombobox)
- **Hybrid Input Architecture:**
  - Integrates an editable single-line `NSTextField` with a right-anchored dropdown trigger button containing `MacSymbol::ChevronUpDown`.
  - Clicking the trigger button or pressing `Alt+Down` opens a floating, viewport-clamped popover list filterable by typed substring.
- **Dimensions Across Control Sizes:**
  - Regular: Frame height $23.0\,\text{pt}$ (intrinsic $20.0\,\text{pt}$), right button width $27.0\,\text{pt}$, left text inset $9.0\,\text{pt}$, corner radius $R = 5.0\,\text{pt}$.
  - Small: Frame height $19.0\,\text{pt}$ (intrinsic $16.0\,\text{pt}$), right button width $20.0\,\text{pt}$, left text inset $7.0\,\text{pt}$, corner radius $R = 4.0\,\text{pt}$.
  - Mini: Frame height $15.0\,\text{pt}$ (intrinsic $13.0\,\text{pt}$), right button width $18.0\,\text{pt}$, left text inset $6.0\,\text{pt}$, corner radius $R = 3.0\,\text{pt}$.
- **Dropdown Popover Metrics:**
  - `intercellSpacing = (3.0, 2.0)\,\text{pt}`, row height `itemHeight = 18.0\,\text{pt}`.
  - Default visible rows `numberOfVisibleItems = 5` yielding standard viewport height of $90.0\,\text{pt}$ before scrolling.

### 3.4 `MacTabView` (`NSTabView`) — `REQ-012-04`
- **Reference:** [Apple Developer Documentation — `NSTabView`](https://developer.apple.com/documentation/appkit/nstabview)
- **Coordinate Model:** Explicitly flipped view (`isFlipped == 1`) where origin $(0, 0)$ is at top-left.
- **Tab Styles:**
  - `NSTopTabsBezelBorder` (`0`):
    - Regular: Tab bar height $24.0\,\text{pt}$. Content insets from `bounds`: top $33.0\,\text{pt}$, bottom $13.0\,\text{pt}$, left/right $10.0\,\text{pt}$. Minimum container size $150.0 \times 36.0\,\text{pt}$.
    - Small: Tab bar height $20.0\,\text{pt}$. Content insets: top $25.0\,\text{pt}$, bottom $13.0\,\text{pt}$, left/right $10.0\,\text{pt}$. Minimum container size $122.0 \times 28.0\,\text{pt}$.
    - Mini: Tab bar height $15.0\,\text{pt}$. Content insets: top $19.0\,\text{pt}$, bottom $13.0\,\text{pt}$, left/right $10.0\,\text{pt}$. Minimum container size $107.0 \times 22.0\,\text{pt}$.
  - `NSNoTabsNoBorder` (`6`):
    - Content rect equals outer bounds with $0.0\,\text{pt}$ insets (`contentRect == bounds`), ideal for programmatically controlled multi-view workflows.

### 3.5 `MacDisclosureGroup` (`NSButton` Disclosure Primitives) — `REQ-012-05`
- **Reference:** [Apple Developer Documentation — `NSButton.BezelStyle`](https://developer.apple.com/documentation/appkit/nsbutton/bezelstyle)
- **Bezel Styles & Kinematics:**
  - `NSBezelStyleDisclosure` (`5` — Classic Triangle):
    - Invariant chevron container size $13.0 \times 13.0\,\text{pt}$ across `Regular`, `Small`, and `Mini`.
    - Horizontal spacing to trailing label: $6.0\,\text{pt}$.
    - Rotation: $0^\circ$ (collapsed, pointing right) $\to 90^\circ$ (expanded, pointing down), animated via `CASpring(mass: 1.0, stiffness: 300.0, damping: 25.0)` settling within $< 150\,\text{ms}$.
  - `NSBezelStyleRoundedDisclosure` (`14` — Rounded Button):
    - Regular: Frame $28.0 \times 26.0\,\text{pt}$ (intrinsic $20.0 \times 20.0\,\text{pt}$).
    - Small: Frame $24.0 \times 21.0\,\text{pt}$ (intrinsic $16.0 \times 16.0\,\text{pt}$).
    - Mini: Frame $19.0 \times 16.0\,\text{pt}$ (intrinsic $13.0 \times 13.0\,\text{pt}$).

### 3.6 `MacCollectionView` (`NSCollectionView` + `NSCollectionViewFlowLayout`) — `REQ-012-06`
- **Reference:** [Apple Developer Documentation — `NSCollectionView`](https://developer.apple.com/documentation/appkit/nscollectionview)
- **Flow Layout Defaults:**
  - Item size: `itemSize = 50.0 × 50.0 pt`.
  - Line spacing: `minimumLineSpacing = 10.0 pt`.
  - Interitem spacing: `minimumInteritemSpacing = 10.0 pt`.
  - Section insets: `sectionInset = (0.0, 0.0, 0.0, 0.0) pt`.
- **Selection & Interaction Oracles:**
  - Selection Ring: Corner radius $R = 6.0\,\text{pt}$, stroke $2.0\,\text{pt}$ in `theme.accent_color()` (`#0064E1` Aqua / `#0059D1` DarkAqua).
  - Rubber-Band Marquee: Dragging across empty grid space creates an animated translucent selection box (`theme.selected_content_background()` with $\alpha = 0.20$ fill and $\alpha = 0.80$ stroke) computing 2D AABB intersection with all item rects.
  - Spatial 2D Navigation: Arrow keys (`Up`, `Down`, `Left`, `Right`) calculate nearest neighbor indices across column count $C = \lfloor (W_{\text{viewport}} + S_{\text{item}}) / (W_{\text{item}} + S_{\text{item}}) \rfloor$.
  - Viewport Culling: All items outside `ui.clip_rect()` are strictly bypassed during layout and draw calls to ensure constant frame budgets at $10{,}000+$ items.

### 3.7 Supplementary Primitives — `REQ-012-07`
- **`MacBox` (`NSBox`):**
  - Primary grouped box: Insets from border `{top: 20.0, bottom: 10.0, left: 9.0, right: 9.0}\,\text{pt}$, corner radius $R = 5.0\,\text{pt}$, stroke $1.0\,\text{pt}$ in `theme.separator_color()`.
  - Separator variant: $1.0\,\text{pt}$ hairline centered inside a $5.0\,\text{pt}$ alignment bounding box.
- **`MacRuleEditor` (`NSRuleEditor`):**
  - Slice row height: $32.0\,\text{pt}$.
  - Action buttons: Add (`[+]`) and Remove (`[-]`) stepper buttons sized $20.0 \times 20.0\,\text{pt}$ with standard AppKit symbol glyphs.

---

## 4. ISO/IEC/IEEE 29119-3 Traceability Matrix

| Requirement ID | Control / Subsystem | Test Case ID | Verification Method | Acceptance Criterion |
| :--- | :--- | :--- | :--- | :--- |
| `REQ-012-01` | `MacPlayerView` | `TC-012-01` | Headless `egui` + Parity | Floating pod matches $481 \times 64\,\text{pt}$ with $R = 16.0\,\text{pt}$; $3.0\,\text{s}$ idle auto-hide; scrubber track $3.5\,\text{pt}$; timecodes match `.SFNS-Regular` $12.0\,\text{pt}$. |
| `REQ-012-02` | `MacDatePicker` | `TC-012-02` | Headless `egui` + `proptest` | Stepper width invariant $19.0\,\text{pt}$; Date+Time Regular $161 \times 28\,\text{pt}$; calendar $139 \times 148\,\text{pt}$; clock $119 \times 119\,\text{pt}$; token selection $R = 2.5\,\text{pt}$. |
| `REQ-012-03` | `MacComboBox` | `TC-012-03` | Headless `egui` | Frame heights $23/19/15\,\text{pt}$; button widths $27/20/18\,\text{pt}$; popover clamped with $18.0\,\text{pt}$ row height and $5$ visible items ($90\,\text{pt}$). |
| `REQ-012-04` | `MacTabView` | `TC-012-04` | Headless `egui` | Top tab bar $24/20/15\,\text{pt}$; content insets top $33/25/19\,\text{pt}$, bottom $13\,\text{pt}$, left/right $10\,\text{pt}$; borderless has $0.0\,\text{pt}$ insets. |
| `REQ-012-05` | `MacDisclosureGroup` | `TC-012-05` | Headless `egui` | Disclosure chevron $13 \times 13\,\text{pt}$ invariant; rounded disclosure $28 \times 26\,\text{pt}$ Regular; `CASpring` rotation smoothly interpolates $0^\circ \to 90^\circ$. |
| `REQ-012-06` | `MacCollectionView` | `TC-012-06` | Headless `egui` + `proptest` | FlowLayout item size $50 \times 50\,\text{pt}$; spacing $10\,\text{pt}$; 2D marquee selection calculates correct AABB; items outside `clip_rect` culled. |
| `REQ-012-07` | Supplementary Primitives | `TC-012-07` | Headless `egui` | `MacBox` content insets $\{20, 10, 9, 9\}\,\text{pt}$; separator $1.0\,\text{pt}$ in $5.0\,\text{pt}$ frame; `MacRuleEditor` slice row $32.0\,\text{pt}$. |
