# Changelog

All notable changes to the `egui-macview` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Native MacBook AppKit differential test (`test_macbook_symbol_search_parity_differential`) verifying search glyph parity against Jaco's physical MacBook Air (`Jacos-Air.home`).

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
- CoreAnimation spring physics (`CASpring`) with closed-form analytical damped harmonic oscillator integration.
- Subpixel snapping algorithm eliminating fractional blur across Windows/Linux display scales.
- Dynamic theme resolution (`MacTheme`, `MacAppearance`) supporting Aqua and DarkAqua appearances.
- W3C DTCG design token catalog parser (`tokens/design_tokens.json`, `tokens/typography.json`).
- Baseline test suite validating token deserialization, layout budgets, and physical spring properties.

[0.3.0]: https://github.com/oversluizen/egui-macview/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/oversluizen/egui-macview/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oversluizen/egui-macview/releases/tag/v0.1.0
