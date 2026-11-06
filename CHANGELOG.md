# Changelog

All notable changes to the `egui-macview` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.0]: https://github.com/oversluizen/egui-macview/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/oversluizen/egui-macview/releases/tag/v0.1.0
