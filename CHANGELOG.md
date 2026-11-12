# Changelog

All notable changes to the `egui-macview` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.10] - 2026-11-12

### Added
- **AppKit `NSSlider.numberOfTickMarks` Ticked Slider Mode (`MacSlider::tick_marks`, `MacSlider::fixed_rect`, `MacSlider::control_size`)**: When `tick_marks > 0`, `MacSlider` renders a `4.0pt` neutral track (`slider_track_bg` / `slider_track_border`) without blue accent fill, `count` evenly spaced vertical tick marks (`1.0x4.0pt` at `3.0pt` offset below the track in `slider_tick_mark`), and an AppKit vertical pill/pointer thumb (`7.0x15.0pt`, `3.5pt` corner radius in `slider_thumb_bg` / `slider_thumb_border`).
- **Complete `MacControlSize` (`Regular`, `Small`, `Mini`) Coverage Across NIB Controls (`MacInputCell`, `MacCheckbox`, `MacRadioButton`, `MacPopUpButton`, `MacButtonCell`)**: Promoted `MacRadioButton` to a first-class AppKit radio button struct (`NSButtonTypeRadio`, circular `14/12/10pt` button with inner dot) and wired `.control_size(MacControlSize)` across `MacButtonCell` (`22/18/15pt` height, `13/11/9pt` font) and `MacPopUpButton` (`22/18/15pt` height, `13/11/9pt` font).
- **AppKit `NSSegmentedControl` `.small_square()` Style & `.show_multi(...)` (`NSSegmentedSwitchTrackingSelectAny`)**: Added `.small_square()`, `.show_multi(ui, active_states)`, `.fixed_rect(Rect)`, and `.control_size(MacControlSize)` to `MacSegmentedControl` for contiguous `2.0pt`-cornered multi-toggle inspector strips with `1.0pt` borders and dividers.
- **Multi-Row `MacTagColorPicker` Inspector Sidebar Layout (`.rows(usize)`, `.dot_size(f32)`, `.spacing(f32, f32)`)**: Added multi-row grid layout (e.g. `.rows(2)` producing 2 rows of 6 columns across the 12 semantic tag swatches) with configurable dot diameter and column/row spacing.
- **AppKit `NSPopover` Container Surface (`MacPopoverSurface`)**: Added `MacPopoverSurface` in `src/surfaces/` rendering a `popover_background` panel with `8.0pt` corner radius and `1.0pt` `popover_border`, supporting `.fixed_rect(Rect)`, `.paint_at(ui, rect)`, `.show(ui, ...)`, and surface-aware vibrancy registration.
- **Repository-Wide `src/` Hardcoding & `MacNibPrimitives` Guard (`tests/zero_hardcoding_and_primitive_tests.rs`, `tests/architecture_guard_tests.rs`)**: Expanded automated COTS guards to scan every subdirectory of `src/` for forbidden `Color32::from_rgb`, `Color32::from_rgba`, `from_hex`, raw `[u8; 3]` color arrays, and `MacNibPrimitives`.

## [0.6.9] - 2026-11-11

### Added
- **AppKit Pro-Inspector Transform Origin Matrix (`MacOriginAnchor`, `MacOriginMatrix`, `OriginMatrixMetrics`)**: Added `3x3` 9-point transform origin / anchor pin grid supporting both `MacOriginAnchor` and `usize` (`from_index`) bindings, `Regular` (`34pt`) / `Small` (`28pt`) / `Mini` (`22pt`) control sizes, `.fixed_rect(Rect)`, pointer click selection, and keyboard arrow-key navigation.
- **macOS Finder / AppKit Tag Color Swatch Picker (`MacTagColorPicker`, `TagColorPickerMetrics`, `MacTagLabelColors`)**: Added 12-slot semantic label swatch picker (`Red`, `Orange`, `Brown`, `Yellow`, `LightGreen`, `DarkGreen`, `Cyan`, `Blue`, `Purple`, `Magenta`, `Gray`, plus `Clear`/`None` cross swatch) supporting `1x12`, `2x6`, and custom column/palette configurations.
- **AppKit `NSBezelStyleSmallSquare` Table Action Bar (`MacTableActionBar`, `MacTableActionResponse`, `TableActionBarMetrics`)**: Added bottom table/list `[ + | - | action ]` strip supporting `.can_add(bool)`, `.can_remove(bool)`, `.with_action_menu(bool)`, and `.fill_remaining_strip(bool)`.

## [0.6.8] - 2026-11-11

### Added
- **Idiomatic `egui::Ui` Surface Propagation & Zero-Allocation Geometric Surface Registry (`MacTheme::register_surface_rect`, `MacTheme::active_surface_at`)**: Propagates closure-scoped container surfaces (`MacBox::show`, `MacCard::show`) natively via `child_ui.visuals_mut().panel_fill` and registers finalized container rects `(rect, surface_bg)` in a per-pass `Copy` array (`PASS_SURFACE_RECTS`) for flat NIB `.fixed_rect(rect)` peer controls.
- **Unified AppKit Baseline & NIB Visual Bezel Helpers (`MacTheme::baseline_text_y`, `MacTheme::clamp_nib_visual_rect`)**: Deduplicated `firstBaselineOffsetFromTop - ascender` baseline anchoring and NIB frame-to-visual bezel clamping into `MacTheme`, wired across `MacLabel`, `MacInputCell`, `MacPopUpButton`, `MacComboBox`, `MacCheckbox`, `MacHelpButton`, and `MacInlineActionButton`, with native `ui.is_enabled()` (`ui.add_enabled_ui(false, ...)`) inheritance.
- **Automated Surface-Aware Vibrancy & Baseline Architecture Guard (`tests/architecture_guard_tests.rs`, `SPEC-001-APPKIT-PRIMITIVES-AND-LAYOUT.md`)**: Anchored container surface awareness and shared baseline/bezel helpers in `SPEC-001 §4` and `test_guard_surface_aware_vibrancy_anchoring`.
- **Complete Legal, Third-Party Font & Trademark Compliance (`LICENSE`, `assets/fonts/CupertinoIcons-LICENSE.txt`, `README.md`, `tests/architecture_guard_tests.rs`)**: Added root `LICENSE` (MIT warranty/liability disclaimer + third-party font notices), `assets/fonts/CupertinoIcons-LICENSE.txt` (Flutter MIT notice), and nominative trademark/non-affiliation disclaimer in `README.md`, guarded by `test_guard_open_source_fonts_only_zero_proprietary_apple_fonts`.

## [0.6.7] - 2026-11-11

### Added
- **AppKit Vibrancy & Surface-Aware Contrast Engine (`plus_darker`, `plus_lighter`, `ensure_wcag_contrast`, `MacTheme::label_on`, `secondary_label_on`, `tertiary_label_on`)**: Added dynamic CoreGraphics/AppKit vibrancy compositing (`kCGBlendModePlusDarker` on light surfaces, `kCGBlendModePlusLighter` on dark surfaces) coupled with automatic binary-search WCAG 2.1 Level AA contrast clamping (`>= 4.5:1` for primary/secondary labels, `>= 3.0:1` for tertiary/disabled labels) against any container surface color.
- **Zero-Allocation Container Surface Scope (`MacTheme::active_surface`, `MacTheme::with_surface`, `MacLabel::resolved_color_on`)**: Wired `MacBox`, `MacCard`, `MacLabel`, `MacDisclosureGroup`, `MacCheckbox`, and `MacInlineActionButton` to automatically resolve vibrant, WCAG-compliant foreground tints against their enclosing surface (`window_background`, `box_background`, or custom container fill) with zero per-frame heap allocations.

## [0.6.6] - 2026-11-11

### Added
- **Zero-Allocation `Inter-SemiBold` Font Lookup (`MacTheme::semibold_font_id`)**: Added `OnceLock<FontFamily>` + per-context `egui::Memory` registration caching on `MacTheme` so `MacLabel`, `MacDisclosureGroup`, and `MacHelpButton` resolve semibold typography without per-frame `Arc<str>` or `Vec<FontFamily>` heap allocations.
- **Size-Calibrated `MacHelpButton` Typography & `MacPopUpButton::accent_badge(bool)`**: Added `small_font_size` (`10.0pt`) and `mini_font_size` (`9.0pt`) tokens to `HelpButtonMetrics` with Semibold `?` rendering, clamped `MacInlineActionButton` visual circle radius inside larger NIB alignment frames, and added `.accent_badge(bool)` to `MacPopUpButton`.

### Fixed
- **MacBook Air Ground-Truth `NSTextFieldSquareBezel` Corner Radius (`tokens/design_tokens.json`)**: Calibrated `text_field` corner radius to physical `macOS Sonoma 14.8.9` `@2x` Retina ground truth (`2.0pt` for `Regular`, `Small`, and `CompactInfobox`; `1.5pt` for `Mini`), distinguishing square text inputs from `5.0pt` rounded `MacPopUpButton` bezels.
- **WCAG 2.1 AA Contrast for Aqua `secondary_label` on `MacBox` (`#5A5A5F`)**: Darkened `aqua.secondary_label` from `#6E6E73` (`4.01:1` on `#E5E5E5`) to `#5A5A5F`, achieving `5.48:1` contrast on `aqua.box_background` (`#E5E5E5`) and `5.84:1` on `aqua.window_background` (`#ECECEC`).
- **Zero-Allocation `MacTheme::tracking_for_font_size`**: Replaced per-call `format!("pt_{}", ...)` heap allocation with a static `const fn tracking_table_key(pt: i32) -> Option<&'static str>` lookup table (`6..=24`).

## [0.6.5] - 2026-11-11

### Added
- **Universal Apple HIG Optical Tracking (`MacTheme::tracking_for_font_size`, `tracked_text_format`, `layout_tracked_galley`)**: Wired `tokens/design_tokens.json` optical tracking (`13pt -> -0.08pt`, `12pt -> 0.0pt`, `11pt -> +0.06pt`, `9pt -> +0.17pt`, `15pt -> -0.23pt`) across all controls (`MacButtonCell`, `MacPopUpButton`, `MacInputCell`, `MacComboBox`, `MacDisclosureGroup`, `MacBox`, `MacTableView`, `MacOutlineView`, `MacSegmentedControl`, `MacCheckbox`, `MacLabel`).
- **Native AppKit Checkbox (`MacCheckbox`, `NSButtonTypeSwitch`)**: Square rounded checkbox (`14x14pt` Regular, `12x12pt` Small, `10x10pt` Mini) with unchecked, checked (3-point vector checkmark `(0.24, 0.52) -> (0.43, 0.72) -> (0.76, 0.28)`), and mixed (`-`) states, `.fixed_rect(Rect)`, and optical tracking.
- **First-Class AppKit Form & Section Label (`MacLabel`, `MacFontWeight`, `MacLabelColor`)**: Non-editable `NSTextField` label supporting `Regular`/`Small`/`Mini` control sizes, `Regular`/`Medium`/`Semibold`/`Bold` font weights, `Primary`/`Secondary`/`Tertiary` semantic label colors, `.alignment(Align)`, `.fixed_rect(Rect)`, and deterministic `firstBaselineOffsetFromTop` anchoring.
- **Circular AppKit Help Button & Inline Action Badge (`MacHelpButton`, `MacInlineActionButton`)**: `20x20pt` circular `NSBezelStyleHelpButton` (`?`) and `16x16pt` inline action badge button (`+`, `-`, `×`, or `MacSymbol`) with `.fixed_rect(Rect)` support.
- **Bundled Open-Source `Inter` (SIL OFL 1.1) + `CupertinoIcons` (MIT) (`setup_symbols_font`, `setup_macview_fonts`)**: Embedded `Inter-Regular.ttf` and `Inter-SemiBold.ttf` alongside `CupertinoIcons.ttf` in `assets/fonts/` with an automated license guard forbidding proprietary Apple fonts.

### Changed
- **`MacInputCell` (`NSTextField` `NSControlSize`, `.fixed_rect(Rect)`, `.text_align(Align)`)**: Defaulted `MacInputCell` to `MacControlSize::Regular` (`22pt` height, `13pt` font, `7pt` horizontal padding), added automatic size detection from `fixed_rect.height()`, added `.text_align(Align)`, and anchored unfocused text rendering to AppKit `firstBaselineOffsetFromTop`.
- **`MacPopUpButton` & `MacComboBox` NIB Visual Bezel Clamping & Accent Badge**: Separated `MacPopUpButton` upper (`⌃`) and lower (`⌄`) chevrons using `theme.label()` stroke color, added the `17pt` right accent button badge to `MacComboBox`, and clamped `25pt`/`23pt` NIB frames to `20pt` visual bezels.
- **`MacBox` Recessed Surface Tokens & `MacDisclosureGroup` Trailing `+` Action**: Added `theme.box_background()` (`#E5E5E5` Aqua / `#232528` DarkAqua), `theme.box_border()` (`#D8D8D8` / `#3A3D42`), `MacBox::fixed_rect(Rect)`, and `MacDisclosureGroup::with_trailing_add_button(bool)` returning `MacDisclosureResponse<R>`.

## [0.6.4] - 2026-11-10

### Added
- **Repository-Wide 100% English & Zero-Leakage Guard (`tests/architecture_guard_tests.rs`, `tests/guard_helpers/mod.rs`)**: Expanded `test_guard_zero_vault_leakage_and_de_ai_ification` to dynamically inspect all `.rs`, `.md`, `.toml`, `.json`, and `.cmd` files across the repository root, `src/`, `examples/`, `tests/`, `specs/`, and `tokens/` with word-boundary non-English (Dutch) detection and self-check verification.

### Fixed
- **100% English Comments, Identifiers & Packaged Files (`CHANGELOG.md`, `specs/SPEC-009-CROSS-PLATFORM-DIFFERENTIAL-HARNESS.md`, `tests/`, `start_gallery.cmd`)**: Removed the legacy Dutch `AUDIT-REFACTORING-REPORT.md` from version control, standardized `Zero Dead Code Guard` terminology (`check_zero_dead_code_in_source`), and replaced the hardcoded path in `start_gallery.cmd` with portable `%~dp0` resolution.

## [0.6.3] - 2026-11-10

### Added
- **IDE Hover `/// # Example` Rustdoc Blocks (`src/navigation/`, `src/surfaces/`, `src/feedback/`, `src/state/`, `src/symbols.rs`)**: Added runnable, doc-tested `/// # Example` blocks onto `MacToolbar`, `MacMenuBar`, `MacCommandPalette`, `MacCanvasFooter`, `MacPopover`, `MacFloatingInfoBox`, `MacContextMenu`, `MacToastManager`, `NSUndoManager`, and `MacSymbol`.
- **`README.md` Rustdoc Intra-Doc Link Guard (`tests/architecture_guard_tests.rs`)**: Added automated architecture guard asserting zero relative `](src/` file links in `README.md` so all API references resolve cleanly on `docs.rs`.

### Fixed
- **Zero Broken Links on `docs.rs` & GitHub (`README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`)**: Converted 62 relative `(src/...)` file links in `README.md` API reference tables to rustdoc intra-doc links (`[`Type`]`), fixed the relative `SPEC-INDEX.md` path in `ARCHITECTURE.md`, and added `[0.4.1]`–`[0.6.3]` version comparison links in `CHANGELOG.md`.

## [0.6.2] - 2026-11-10

### Added
- **Standalone `eframe` Quickstart Application (`README.md`)**: Complete, copy-pasteable `src/main.rs` desktop application ("Your First App in 60 Seconds") demonstrating `eframe::App` wiring, `setup_symbols_font` registration, `MacTheme::apply_to_egui`, `MacToolbar`, `MacSplitView`, `MacBox`, and `MacForm`.
- **Apple HIG Control Decision Guide (`README.md`)**: Practical "When to Use Which Control" comparison tables for selection controls (`MacSegmentedControl` vs `MacPopUpButton` vs `MacComboBox`), data views (`MacTableView` vs `MacOutlineView` vs `MacCollectionView`), layout containers (`MacSplitView` vs `MacTabView` vs `MacDisclosureGroup` vs `MacBox`), and numeric inputs (`MacSlider` vs `MacStepper` vs `MacCircularSlider` vs `MacInputCell::numeric`).
- **Troubleshooting & FAQ Guide (`README.md`)**: Doc-tested answers and code recipes for symbol font setup, dynamic `Aqua` / `DarkAqua` / `Anthracite` theme switching, avoiding widget ID collisions inside loops, and battery-saving zero-idle-repaint behavior.
- **IDE Hover `/// # Example` Rustdoc Blocks (`src/controls/`, `src/layout/`)**: Added runnable, doc-tested `/// # Example` blocks directly onto `MacPlayerView`, `MacDatePicker`, `MacComboBox`, `MacTabView`, `MacDisclosureGroup`, `MacDisclosureButton`, `MacCollectionView`, `MacBox`, `MacRuleEditor`, `MacCircularSlider`, `MacRatingIndicator`, `MacPathControl`, and `MacForm`.
- **Visual Mermaid Architecture Diagrams (`ARCHITECTURE.md`)**: Added flowcharts illustrating the Design Token $\to$ `MacTheme` $\to$ Widget resolution pipeline and the 3-Phase Widget Lifecycle (`compute_geometry` $\to$ `handle_input` $\to$ `paint`).

## [0.6.1] - 2026-11-10

### Added
- **AppKit Predicate Rule Editor (`MacRuleEditor`, `MacRuleRow`, `REQ-012-07`)**: Multi-row slice predicate editor (`NSRuleEditor` equivalent) with `32.0 pt` row height, `8.0 pt` clause spacing, interactive slice criteria pills, and `20.0 x 20.0 pt` `[+]` / `[-]` row buttons.
- **Full AVKit Transport, Audio & Speed Controls (`MacPlayerView`, `REQ-012-01`)**: Complete 3-phase pipeline (`compute_geometry`, `handle_input`, `paint`) across `Floating` (`481.0 x 64.0 pt` pod with Mute button, Volume slider, 15s Skip Back/Forward, 1-Frame Step Back/Forward, Speed pill `[0.5x..2.0x]`, AirPlay badge, and buffered/elapsed scrubber), `Inline` (`24.0 pt` bottom bar), and `Minimal` (`29.0 x 18.0 pt` pill).
- **SPEC-012 AppKit & Battery Parity Test Suite (`tests/spec012_appkit_and_battery_parity_tests.rs`)**: Comprehensive COTS tests covering HUD transport/audio/speed controls, stationary-cursor auto-hide, direct segment token clicks, analog clock ticks & click-to-set-minute, combo box text inset & exact-match filtering, centered segmented tab pill bar, $O(1)$ collection view row culling, and `MacRuleEditor` mutations.

### Fixed
- **Battery Efficiency & Zero Idle Allocations (`SPEC-012`)**:
  - `MacPlayerView`: Fixed stationary-cursor auto-hide (`delta != Vec2::ZERO`) so the floating HUD fades out after `3.0s` via a single `request_repaint_after_secs` wakeup and settles with zero continuous repaints; isolated multi-instance widget IDs (`response.id`).
  - `MacComboBox`: Eliminated per-frame string/vector allocations when closed (`!is_open`), applied size-specific left `text_inset` (`9.0/7.0/6.0 pt`), preserved sibling items on exact match, and restricted `scroll_to_me` to keyboard navigation frames.
  - `MacCollectionView`: Upgraded both rendering and rubber-band marquee selection to $O(1)$ row-range slicing (`first_row..last_row`) and added keyboard focus acquisition on item click.
  - `MacDatePicker`: Replaced per-frame heap `Vec<String>` formatting in `TextFieldAndStepper` with a stack-allocated `[SegToken; 11]` buffer, added direct mouse click segment selection, added the `16.0 pt` weekday row (`S M T W T F S`), and added 12-hour analog clock tick marks + click-to-set-minute.
  - `MacTabView` & `MacDisclosureGroup`: Centered the top `NSTopTabsBezelBorder` segmented pill bar straddling the top bezel with MacBook minimum container bounds (`150x36`, `122x28`, `107x22 pt`), and gated disclosure `CASpring` animations on `is_rect_visible && is_window_active`.
  - `MacTheme::resolve_or_default`: Eliminated 26 runtime `egui::Context::default()` heap allocations across control metric getters by resolving directly against the `OnceLock<Arc<MacDesignTokens>>` singleton.
  - `MacBox` & `MacTabView`: Fixed child `UiBuilder::id_salt` generation via `ui.next_auto_id()` to prevent widget ID collisions when multiple containers are rendered in the same scope, and ensured background bezel shapes paint behind child widgets.
- **English Documentation & Copy-Paste Quickstart Guide (`README.md`, `ARCHITECTURE.md`, `src/lib.rs`)**: Rewrote crate documentation in clear, approachable English with 7 copy-pasteable doc-tested examples covering all 35+ controls, layouts, and themes.

## [0.6.0] - 2026-11-10

### Added
- **Advanced Media, Scheduling & Grid Controls (`SPEC-012`)**:
  - `MacPlayerView`: Authentic AVKit media player suite (`AVPlayerView`) featuring QuickTime floating HUD transport pod (`AVPlayerViewControlsStyleFloating` with volume slider, mute toggle, 15s skip, 1-frame step, playback rate selector `[0.5x..2.0x]`, AirPlay badge, and buffered/elapsed timeline scrubber), inline bottom bar (`AVPlayerViewControlsStyleInline`), minimal transport pill (`AVPlayerViewControlsStyleMinimal`), and battery-efficient 3.0s stationary-cursor auto-hide (`REQ-012-01`).
  - `MacDatePicker`: Dual date/time picker supporting direct segment-clickable text-field with integrated stepper (`19.0 pt` width, `9.0 pt` chevron pitch, zero heap allocations per frame), interactive 12-hour analog clock face (`122x123 pt`), and month calendar matrix grid (`139x148 pt` with `24.0 pt` nav header + `16.0 pt` weekday row + `6x7` day matrix) across `Regular`, `Small`, and `Mini` sizes (`REQ-012-02`).
  - `MacComboBox`: Hybrid editable text field with size-calibrated left text inset (`9.0/7.0/6.0 pt`), right-anchored dropdown chevron trigger, and viewport-clamped filterable popover list (`18.0 pt` row height, 5 visible items, `90.0 pt` viewport, zero allocations when closed) (`REQ-012-03`).
  - `MacTabView`: Multi-pane container with flipped coordinate system (`isFlipped == 1`), centered top segmented pill bar straddling the top bezel line (`NSTopTabsBezelBorder`), MacBook minimum container dimensions (`150x36`, `122x28`, `107x22 pt`), and borderless card switching (`NSNoTabsNoBorder`) (`REQ-012-04`).
  - `MacDisclosureGroup` & `MacDisclosureButton`: Collapsible content container with `13.0 pt` triangular disclosure chevron, visibility/focus-gated `CASpring` rotation physics, and standalone rounded pill disclosure button across `Regular`, `Small`, and `Mini` sizes (`REQ-012-05`).
  - `MacCollectionView`: High-performance 2D grid layout engine (`NSCollectionViewFlowLayout`) with $O(1)$ row-range viewport culling (`ui.clip_rect()`), $O(1)$ row-range rubber-band marquee multi-selection, and 2D spatial keyboard navigation (`REQ-012-06`).
  - `MacBox` & `MacRuleEditor`: Grouping container (`NSBox`) with title badge, `5.0 pt` corner radius, standardized AppKit content insets (`top: 20.0 pt`, `bottom: 10.0 pt`, `left/right: 9.0 pt`), and multi-row predicate slice rule editor (`NSRuleEditor` with `32.0 pt` rows and `20x20 pt` `[+]`/`[-]` buttons) (`REQ-012-07`).
- **Interactive 6-State Gallery Showcase (`examples/macview_gallery.rs`, `examples/gallery/`)**: Added State 6 (`GalleryScreen::AdvancedControls`) with live interactive demonstrations of all SPEC-012 controls and copyable Rust snippet generation.
- **SPEC-012 Integration & Parity Test Suites (`tests/spec012_advanced_controls_tests.rs`, `tests/spec012_appkit_and_battery_parity_tests.rs`)**: Comprehensive COTS integration tests covering MacBook AppKit/AVKit geometry, interaction state machines, $O(1)$ viewport culling, and stationary-cursor battery efficiency.

### Changed
- Bumped crate version to `0.6.0` in `Cargo.toml`, `src/lib.rs`, and `tokens/design_tokens.json`.
- Promoted `SPEC-012` and `SPEC-INDEX` to `Accepted / Normative` (`0.6.0`).

## [0.5.2] - 2026-11-10

### Added
- **Circular Dial & Star Rating Controls (`MacCircularSlider`, `MacRatingIndicator`)**: Added authentic continuous radial dial knob with CASpring physics and discrete/continuous 5-star rating control with precomputed unit-star geometry (`tests/battery_efficiency_and_avkit_tests.rs`).
- **Live AVKit Audio/Video Studio & Zero Idle Repaint**: Added real-time 30 FPS video canvas and multi-channel audio synthesis, unbuffered/buffered inspection, out-of-trim playhead navigation, and zero-repaint battery efficiency when unfocused or minimized.
- **Finder Navigation & Directory Browsing**: Added authentic directory browsing with pinned parent directory (`..`), Windows drive root navigation (`C:\`, `D:\`), and `Cmd+Down`/`Enter` row activation (`tests/appkit_finder_and_table_tests.rs`).
- **Interactive Rust Snippet Generator (`examples/gallery/snippet.rs`)**: Added copyable, self-contained Rust code generator for all 5 gallery states with toast notification feedback.
- **SPEC-012 Normative Specification**: Formal specification for Milestone 0.6.0 (`specs/SPEC-012-ADVANCED-MEDIA-SCHEDULING-AND-GRID-CONTROLS.md`) covering `MacPlayerView`, `MacDatePicker`, `MacComboBox`, `MacTabView`, `MacDisclosureGroup`, `MacCollectionView`, `MacBox`, and `MacRuleEditor`.

### Fixed
- **Table View Focus & Arrow Key Interception**: Gated keyboard arrow navigation on explicit focus/hover to prevent key event theft across overlapping widgets.
- **Scrubber Playhead Invariants**: Allowed playhead inspection outside trim range during scrubbing when clamp is disabled while preserving strict trim bounds on playback.
- **Color Well Salt Deduplication**: Added unique ID salting to prevent color picker popup conflicts across duplicate property inspector rows.

## [0.5.1] - 2026-11-08

### Added
- **Automated Architecture Fitness Guards (`tests/architecture_guard_tests.rs`)**: Added Windows Desktop GUI Subsystem & Safe `eframe` Startup Guard (`#![cfg_attr(not(test), windows_subsystem = "windows")]`, `Renderer::Glow`, `InstanceFlags::empty()`, `system_font_fallback: false`, `persist_window: false`), Zero Dead Code Guard (`allow(dead_code)` forbidden in `src/`), Zero Unused Dependencies Guard, and Central State-Invariant Guard (`NSUndoManager::validate_invariants`).
- **Cold-Start, Zero-Idle-Repaint & State-Invariant Smoke Suite (`tests/runtime_hygiene_and_invariants_tests.rs`)**: Headless cold-start + first-frame latency verification (`< 500ms`), zero idle repaint leak verification (`repaint_delay > Duration::ZERO` after 2 rest frames) across all 5 `GalleryScreen` states (`Aqua` & `DarkAqua`) and `StudioApp`, `MacTrimScrubber` bounds invariants, `MacTableView` filtered selection invariants, and bit-exact `Action -> Undo -> Redo` roundtrips on `NSUndoManager`.

### Changed
- **Single-Pass `Arc<Galley>` Reuse & Consolidated `egui::Memory` State**: Consolidated `MacScrollView` (`ScrollViewTempState`) and `MacInputCell` (`InputCellTransactionState`) into single `IdTypeMap` lookups per frame, and reused laid-out `Arc<Galley>`s (`Color32::PLACEHOLDER`) across `MacButtonCell`, `MacSegmentedControl`, `MacPopUpButton`, `MacToolbar`, `MacPathControl`, `MacTokenField`, `MacMenuBar`, `submenu.rs`, and `MacContextMenu`.
- **Safe Windows Hybrid-GPU `eframe` Startup (`examples/*.rs`)**: Configured `eframe::Renderer::Glow`, `wgpu::InstanceFlags::empty()`, `system_font_fallback: false`, and `persist_window: false` across all GUI examples to prevent DX12/Vulkan validation TDR (`LiveKernelEvent 141`) and font-indexing main-thread freezes.

### Fixed
- **`MacTableView` Selection Pruning on Row Count Reduction**: Pruned out-of-bounds `selected_rows` indices (`self.selection.retain(|&idx| idx < self.row_count)`) when table filtering reduces `row_count`.

## [0.5.0] - 2026-11-08

### Added
- **AppKit Progress Indicator (`MacProgressIndicator`, `MacProgressStyle`, `MacProgressControlSize`)**: Determinate/indeterminate horizontal progress bar (`6.0 pt` Regular, `4.0 pt` Small/Mini track) and 12-spoke stepped circular activity spinner (`32.0 pt` Regular, `16.0 pt` Small, `12.0 pt` Mini at 12 FPS) with optional status caption and percentage readout (`SPEC-011`, `REQ-011-01`).
- **AppKit Multi-Column Data Table (`MacTableView`, `MacTableColumn`, `MacSortDirection`, `MacTableViewResponse`)**: Multi-column table view with `28.0 pt` `NSTableHeaderView`, clickable column sort direction toggles (`ChevronUp` / `ChevronDown`), alternating row backgrounds (`alternatingContentBackgroundColors`), single/Shift-range/Cmd-toggle multi-row selection (`BTreeSet<usize>`), and consumed `ArrowUp`/`ArrowDown` keyboard navigation (`REQ-011-02`).
- **AppKit Pill Tag Input (`MacTokenField`)**: Interactive capsule token input field (`16.0 pt` inner capsules with `8.0 pt` corner radius inside a `22.0 pt` min-height container) with automatic comma/Enter tokenization, per-pill `Xmark` removal, `Backspace` pop on empty buffer, multi-line flow wrapping, and optional live `N / M` character counter (`REQ-011-03`).
- **AppKit Capacity & Audio Meter (`MacLevelIndicator`, `MacLevelIndicatorStyle`, `MacLevelOrientation`)**: Continuous and discrete segmented capacity/audio level meter in horizontal or vertical orientation with semantic threshold coloring (`systemGreen`, `systemYellow`, `systemRed`) and optional peak-hold tick marker (`REQ-011-04`).
- **AVKit / QuickTime Trim Scrubber (`MacTrimScrubber`, `MacTimelineZone`, `MacTrimScrubberResponse`)**: Media timeline trim bar with waveform envelope bars, named timeline zone overlays, outside-trim dimming scrims (`rgba(0, 0, 0, 0.58)`), yellow selection frame (`systemYellow`, `2.0 pt` stroke) with left/right slotted grab handles (`12.0 pt` wide), and draggable playhead cursor with floating `MM:SS.d` timecode badge (`REQ-011-05`).
- **AppKit Breadcrumb Path Bar (`MacPathControl`, `MacPathComponent`, `MacPathControlResponse`)**: Interactive file/folder breadcrumb bar (`22.0 pt` height) with vector folder/document icons, `ChevronRight` separators, per-segment click selection, and optional trailing `Choose...` button (`REQ-011-06`).
- **Numeric Arrow-Key Stepping (`MacInputCell::numeric`)**: `NSTextField` + `NSNumberFormatter` keyboard stepping (`ArrowUp`/`ArrowDown` `±step`, `Shift` `±10×`, `Cmd/Ctrl` `±100×`) with range clamping, decimal precision preservation, and transactional `Escape` rollback.
- **Modular 5-State Studio Gallery (`examples/macview_gallery.rs`, `examples/gallery/`)**: Interactive 5-state studio showcase covering `v0.1`–`v0.5` (`Vector & Inspector Studio`, `Cupertino Symbols & Layout Lab`, `Command, Menu & State Integrity Lab`, `Finder & Publishing Data Browser`, `AVKit Media & Audio Timeline Studio`) with dynamic `ViewportCommand::Decorations` switching and 8-directional frameless window resizing.
- **Live MacBook Air Ground-Truth & Differential Parity Suite (`SPEC-011`)**: Extracted PyObjC metrics and offscreen bitmap renders (`appkit_studio_v05_ground_truth.json` + 12 Aqua/DarkAqua PNG golden masters) from physical macOS hardware (`ssh macbook`) and added `studio_controls_v05_tests.rs`.

### Changed
- Bumped crate version to `0.5.0` in `Cargo.toml` and `tokens/design_tokens.json`.
- Added `specs/SPEC-011-STUDIO-AND-AVKIT-CONTROLS.md` and synchronized `specs/SPEC-INDEX.md` to `0.5.0`.

## [0.4.1] - 2026-11-08

### Added
- **Unified Window Toolbar (`MacToolbar`, `SPEC-005`)**: First-class library widget with 3-zone AppKit layout, configurable `WindowControlsStyle` (`AppleTrafficLights` with cluster-wide hover glyphs, `WindowsCaptionButtons` with 46pt Fitts's Law caption buttons, `NativeHost`), double-click maximize, and background window dragging.
- **Floating Compact HUD Palette (`MacFloatingInfoBox`, `SPEC-006 §2.2`)**: First-class `178x63pt` translucent utility HUD palette with `22pt` draggable header and two `18pt` rows rendered inside an `Order::Foreground` `egui::Area` with full click, drag, and scroll isolation.
- **Context-Aware Theme Fallback (`MacTheme::from_ctx`)**: `MacTheme::apply_to_egui` stores the active theme in `egui::Context` data so any widget omitting `.theme(...)` inherits the active `Aqua` / `DarkAqua` appearance automatically.
- **Expanded W3C DTCG Token Coverage**: Full token coverage for `navigation` (`footer`, `palette`, `menu`), `feedback` (`context_menu`, `toast`), `surfaces` (`hud`), and `theme` with zero hardcoded magic numbers or colors across all UI modules.

### Fixed
- **DTCG Color Parser (`parse_color32`)**: Fixed 8-digit hex (`#RRGGBBAA`) and `rgba(r, g, b, a)` parsing to use `Color32::from_rgba_unmultiplied` instead of `from_rgba_premultiplied`.
- **`MacInputCell` Transactional Buffer & Tab FSM**: Implemented `egui::Memory`-backed transactional edit buffer (snapshot on focus, commit on `Enter`/`Tab`/blur, rollback on `Escape`) and eliminated same-frame `Tab`/`Shift+Tab` domino cascades via `consume_key` and `.lock_focus(true)`.
- **`MacSearchField` Paint Z-Order**: Painted pill background, border, focus ring, and search icon prior to `TextEdit::singleline` so typed text and cursor are never occluded.
- **Foreground & Click/Scroll Isolation**: Prevented click and scroll-wheel pass-through across `MacPopUpButton`, `MacPanel`, `MacPopover`, `MacFloatingInfoBox`, and `MacOutlineView` trailing accessories.
- **Spring Micro-Animations & Menu Guards**: Upgraded `MacOutlineView` disclosure chevrons, `MacSegmentedControl` active pills, and `MacToggle` thumbs to evaluate `CASpring` physics; guarded `MacMenuBar::handle_shortcuts` when `ctx.wants_keyboard_input()` is active.

### Removed
- Removed empty `MacFlexView` stub in favor of `MacRow` and `MacStackView`.

## [0.4.0] - 2026-11-07

### Added
- **State Integrity (`NSUndoManager<T>`)**: Bounded ringbuffer transaction stack ($C = 50$) with cyclic state preservation, capacity limits, non-destructive peek inspection, and full rollback exactness verified with `proptest`.
- **Context Menus (`MacContextMenu`)**: Viewport-boundary clamped (`clamp_menu_rect`) secondary-click context popover guaranteeing zero offscreen clipping, full keyboard navigation (Arrow Up/Down, Enter/Space, Escape), and automatic dismissal on click outside bounds.
- **Animated Micro-Feedback (`MacToast` & `MacToastManager`)**: Floating pill toast notification banners driven by `CASpring` entry/exit physics (`response = 0.35s`, `damping_ratio = 0.85`) with 2.5s auto-dismiss timer and semantic levels (`Info`, `Success`, `Warning`, `Error`).
- **Interactive Gallery Integration (`macview_gallery`)**:
  - Live transactional state undo/redo tracking of card model changes (rotations, scales, skews, dimensions, colors) with global shortcuts (`⌘Z` / `Ctrl+Z`, `⌘⇧Z` / `Ctrl+Shift+Z` / `Ctrl+Y`) and command palette integration.
  - Interactive secondary-click context menu on the canvas vector card with actions: `Reset Transform`, `Center Card`, `Fill Width`, `Toggle Proportions`, and `Copy Style`.
  - Micro-feedback toast notifications for user interactions (`"Undone"`, `"Redone"`, `"Preset applied"`, `"Transform reset"`, `"Style copied"`).
- **Normative Anchoring**: Promoted `SPEC-008` (State Integrity & Feedback) and `SPEC-008` (Symbols) to Accepted / Normative release status.

### Changed
- Bumped crate version to `0.4.0` in `Cargo.toml`.
- Bumped W3C DTCG `tokens/design_tokens.json` `$version` to `0.4.0`.
- Synchronized specification suite index and capability matrix in `specs/SPEC-INDEX.md`.

## [0.3.1] - 2026-11-06

### Added
- `MacSymbol`: Authentic monochrome Cupertino vector symbols (`Search`, `ChevronRight`, `ChevronLeft`, `ChevronDown`, `ChevronUp`, `SidebarLeading`, `Gear`, `Sliders`, `Checkmark`, `Xmark`, `Plus`, `Minus`, `ArrowClockwise`, `ArrowCounterClockwise`) with embedded `CupertinoIcons.ttf` (MIT) and idempotent `setup_symbols_font` registration.
- `MacForm`: 2-column AppKit form container wrapping `egui::Grid` with token-driven spacing, trailing labels, and leading controls.
- `SPEC-008`: AppKit Symbols & Iconography specification.
- Native MacBook AppKit differential test (`test_macbook_symbol_search_parity_differential`) verifying search glyph parity against physical Apple Silicon MacBook Air hardware (macOS Sonoma 14.8.9).

### Fixed
- Replaced host emoji (`"🔍"`) in `macview_gallery` with crisp monochrome vector `MacSymbol::Search`.
- Header & Footer layout collisions during fractional zoom scaling (`Ctrl` + `+`).
- Replaced flat blue popover button with authentic `MacButtonCell`.
- Docs.rs crate documentation embedding via `#![doc = include_str!("../README.md")]`.
- Cross-platform Windows 11 & desktop keywords in `Cargo.toml`.

## [0.3.0] - 2026-11-06

### Added
- Mathematical specifications and implementation oracles for `MacBackgroundStyle` dynamic contrast engine (Light vs Dark) adhering to WCAG 2.1 Level AA relative luminance formula ($L = 0.2126R + 0.7152G + 0.0722B$) with an exact $L \le 0.179$ crossover threshold guaranteeing $\ge 4.5:1$ contrast across all dynamic accents and container fills.
- `MacMenuBar` desktop navigation system with pure data-driven `NSMenu` / `NSMenuItem` trees, cascading submenu popovers, and global shortcut accelerators (`MacKeyboardShortcut`).
- `MacCanvasFooter` deterministic $24\,\text{pt}$ layout container (`CANVAS_FOOTER_HEIGHT`) providing breadcrumb trails, coordinates readouts, and interactive zoom presets (`50% | 100% | 200%`).
- `MacCommandPalette` floating Spotlight modal HUD ($480 \times 280\,\text{pt}$) with real-time fuzzy subsequence scoring, query filtering, keyboard arrow navigation, and `Escape` clearing mechanics.
- Promoted `SPEC-007` (Desktop Navigation, Canvas Footer & Spotlight Command Palette) from draft to official released specification status.

### Changed
- Bumped crate version to `0.3.0`.
- Polished desktop navigation controls and layout integration in `examples/macview_gallery.rs`.
- Synchronized specification suite index and roadmap matrix in `SPEC-INDEX.md` marking Milestone 0.3.0 as Released.

## [0.2.0] - 2026-11-06

### Added
- Modular specification suite under `specs/` (`SPEC-INDEX.md`, `SPEC-001` through `SPEC-008`).
- Synchronized versioning across crate, design tokens, and specification suite (`0.2.0`).
- Exported crate version constant `egui_macview::VERSION`.
- `MacInputCell::with_id` constructor preventing double-hashing and automatically requesting focus on cell click.
- Extended AppKit controls: `MacColorWell`, `MacOutlineView`, `MacPopUpButton`, `MacSearchField`, `MacSlider`, `MacStepper`, `MacThumbnailCell`.
- Generic layout containers: `MacScrollView` (with zero-gutter overlay scrollbars and auto-fade), `MacStackView` (with explicit bounding constraints and hairline dividers), and `MacSplitView` (multi-pane with draggable divider).
- Desktop navigation & command palette: `NSMenu`, `NSMenuItem`, `MacMenuBar`, `MacCanvasFooter` (24pt layout budget), `MacCommandPalette` (Cmd+K / Ctrl+K).
- State integrity & feedback: `NSUndoManager` bounded ringbuffer transaction stack with cyclic preservation, `MacContextMenu` boundary clamping, and animated `MacToast` notifications.
- `WindowControlsStyle` strategy (`AppleTrafficLights`, `WindowsCaptionButtons`, `NativeHost`) with subpixel vector rendering and Fitts's law compliance.
- 3-tier inspector architecture presets: Compact Pro HUD Palette (178x63pt), Default NIB Inspector (260pt), and Expanded Pro Inspector (380pt multi-deck).
- Comprehensive property-based test suites and golden master parity differential tests.
- High-assurance architecture guards enforcing zero magic numbers, zero hardcoded colors, modularity ceilings (<400 eSLOC), and domain agnosticism.

### Changed
- Bumped crate version to `0.2.0`.
- Updated W3C DTCG `tokens/design_tokens.json` `$version` to `0.2.0`.
- Upgraded `examples/macview_gallery.rs` to display dynamic crate version `v0.2.0` in the window titlebar and showcase the complete AppKit control suite.

## [0.1.0] - 2026-11-05

### Added
- Initial baseline release of `egui-macview`.
- Core AppKit primitives: `MacButtonCell`, `MacInputCell`, `MacSegmentedControl`, `MacToggle`.
- Fundamental layout containers: `MacFixedView`, `MacFlexView`, `MacRow`.
- CoreAnimation spring physics (`CASpring`) with smooth Apple-style spring dynamics.
- Subpixel snapping algorithm eliminating fractional blur across Windows/Linux display scales.
- Dynamic theme resolution (`MacTheme`, `MacAppearance`) supporting Aqua and DarkAqua appearances.
- W3C DTCG design token catalog parser (`tokens/design_tokens.json`, `tokens/typography.json`).
- Baseline test suite validating token deserialization, layout budgets, and physical spring properties.

[0.6.8]: https://github.com/oversluizen/egui-macview/compare/v0.6.7...v0.6.8
[0.6.7]: https://github.com/oversluizen/egui-macview/compare/v0.6.6...v0.6.7
[0.6.6]: https://github.com/oversluizen/egui-macview/compare/v0.6.5...v0.6.6
[0.6.5]: https://github.com/oversluizen/egui-macview/compare/v0.6.4...v0.6.5
[0.6.4]: https://github.com/oversluizen/egui-macview/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/oversluizen/egui-macview/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/oversluizen/egui-macview/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/oversluizen/egui-macview/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/oversluizen/egui-macview/compare/v0.5.2...v0.6.0
[0.5.2]: https://github.com/oversluizen/egui-macview/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/oversluizen/egui-macview/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/oversluizen/egui-macview/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/oversluizen/egui-macview/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/oversluizen/egui-macview/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/oversluizen/egui-macview/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/oversluizen/egui-macview/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/oversluizen/egui-macview/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oversluizen/egui-macview/releases/tag/v0.1.0
