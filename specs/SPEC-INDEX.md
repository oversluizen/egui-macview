# SPEC-INDEX: `egui-macview` Specification Suite Index & Version Matrix

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-INDEX |
| **Document Title** | Specification Suite Index & Version Matrix |
| **Document Version** | `0.6.6` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.6.6 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: Monolithic Architecture Specification |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Decomposed Modular Specification Suite Synchronized with Crate v0.2.0 |
| `0.4.0` | 2026-11-07 | Architecture Team | Milestone 0.4.0 Release: State Integrity (NSUndoManager), Context Menus, Micro-Feedback & Gallery Integration |
| `0.4.1` | 2026-11-08 | Architecture Team | Milestone 0.4.1 Consolidation & Hardening Release: Zero Hardcoding, Transactional VC State Machines, First-Class `MacToolbar` & `MacFloatingInfoBox`, and Spec ID Disambiguation |
| `0.5.0` | 2026-11-08 | Architecture Team | Milestone 0.5.0 Release: Studio & AVKit Media, Timeline and Publishing Controls (`SPEC-011`) |
| `0.5.1` | 2026-11-08 | Architecture Team | Milestone 0.5.1 Hardening Release: Automated Architecture Fitness Guards, Cross-Platform GPU Hardening, Single-Pass Galley Reuse & Runtime Invariants |
| `0.5.2` | 2026-11-10 | Architecture Team | Milestone 0.5.2 Release: Circular Dial, Star Rating, Audio/Video Gallery, Finder Root Navigation & Registration of `SPEC-012` |
| `0.6.0` | 2026-11-10 | Architecture Team | Milestone 0.6.0 Release: Advanced Media, Scheduling, and Grid Controls (`SPEC-012`), 6-State Studio Gallery & Release 0.6.0 |
| `0.6.1` | 2026-11-10 | Architecture Team | Milestone 0.6.1 Release: Full `MacRuleEditor` (`REQ-012-07`), Complete AVKit Transport/Audio/Speed Controls, Zero Idle Allocations & AppKit/Battery Parity Suite |
| `0.6.2` | 2026-11-10 | Architecture Team | Milestone 0.6.2 Release: Comprehensive Developer Documentation, HIG Control Decision Guide, Standalone `eframe` Quickstart, IDE Hover Doc-Tests & Architecture Diagrams |
| `0.6.3` | 2026-11-10 | Architecture Team | Milestone 0.6.3 Release: `docs.rs` Intra-Doc Link Parity (62 Table Links), Navigation/Surface/Feedback/State/Symbol Hover Examples & Link Guard |
| `0.6.4` | 2026-11-10 | Architecture Team | Milestone 0.6.4 Release: Repository-Wide 100% English Comment & Prose Guard, Removal of Legacy Dutch Audit File & Portable Launcher Script |
| `0.6.5` | 2026-11-11 | Architecture Team | Milestone 0.6.5 Release: AppKit NIB Parity (`MacCheckbox`, `MacLabel`, `MacHelpButton`, `MacInlineActionButton`), Universal Optical Tracking, Cross-Platform Baseline Anchoring & Bundled Open-Source `Inter` Font |
| `0.6.6` | 2026-11-11 | Architecture Team | Milestone 0.6.6 Release: MacBook Air `NSTextFieldSquareBezel` Corner-Radius Parity (`2.0pt`/`1.5pt`), WCAG 2.1 AA Aqua `secondary_label` Contrast Hardening (`#5A5A5F`), `MacPopUpButton::accent_badge(bool)` & Zero-Allocation Typography Lookup |

---

## 1. Specification Suite Structure

The `egui-macview` specification suite provides the formal normative requirements, mathematical foundations, MVC abstractions, and test oracles for the entire crate. Every specification in this directory runs strictly in lockstep with the crate release version (`0.6.6`).

| Spec ID | Document Title | Version | Status | Primary Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| [**SPEC-001**](SPEC-001-DESIGN-TOKENS-AND-WCAG.md) | Design Tokens, Optical Tracking & WCAG AA Contrast | `0.2.0` | Accepted | `tests/wcag_contrast_and_visuals_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs` |
| [**SPEC-002A**](SPEC-002-DYNAMIC-COLOR-AND-TOKENS.md) | Dynamic Color, Tokens & MacBackgroundStyle Contrast Engine | `0.2.0` | Accepted | `tests/wcag_contrast_and_visuals_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs` |
| [**SPEC-002B**](SPEC-002-CONTROLS-AND-MICRO-ANIMATIONS.md) | Authentic AppKit Controls & Micro-Animation Oracles | `0.2.0` | Accepted | `tests/appkit_primitives_tests.rs`, `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-003**](SPEC-003-LAYOUT-ENGINES.md) | Deterministic Layout Engines & Subpixel Snap Rasterization | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/appkit_primitives_tests.rs` |
| [**SPEC-004**](SPEC-004-CORE-ANIMATION-PHYSICS.md) | CoreAnimation Spring Physics Runtime (`CASpring`) | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/macbook_parity_differential_tests.rs` |
| [**SPEC-005**](SPEC-005-WINDOW-FRAMING-AND-CONTROLS.md) | Window Framing, Drag Handling & Configurable Caption Controls | `0.2.0` | Accepted | `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-006**](SPEC-006-INSPECTOR-ARCHITECTURE.md) | Three-Tier Inspector Architecture & Floating Surfaces | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-007**](SPEC-007-DESKTOP-NAVIGATION-AND-PALETTE.md) | Desktop Navigation, Canvas Footer & Spotlight Command Palette | `0.3.0` | Accepted | `tests/menu_and_palette_tests.rs` |
| [**SPEC-008A**](SPEC-008-APPKIT-SYMBOLS-AND-ICONS.md) | AppKit Vector Symbols, Iconography & CupertinoIcons Engine | `0.4.0` | Accepted / Normative | `tests/symbol_tests.rs` |
| [**SPEC-008B**](SPEC-008-STATE-INTEGRITY-AND-FEEDBACK.md) | State Integrity, Context Menus & Micro-Feedback Architecture | `0.4.0` | Accepted / Normative | `tests/undo_and_feedback_tests.rs` |
| [**SPEC-009**](SPEC-009-CROSS-PLATFORM-DIFFERENTIAL-HARNESS.md) | Cross-Platform Parity and Remote Differential Testing Harness | `0.3.0` | Accepted / Normative | `tests/macbook_parity_differential_tests.rs` |
| [**SPEC-010**](SPEC-010-INTERACTIVE-GALLERY-CONTROLS-AND-ICONOGRAPHY.md) | Interactive Gallery Controls, Bidirectional State Binding & Iconography Parity | `0.3.1` | Accepted / Normative | `tests/micro_animations_and_hud_tests.rs`, `tests/symbol_tests.rs` |
| [**SPEC-011**](SPEC-011-STUDIO-AND-AVKIT-CONTROLS.md) | Studio & AVKit Media, Timeline and Publishing Controls | `0.5.0` | Accepted / Normative | `tests/studio_controls_v05_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs` |
| [**SPEC-012**](SPEC-012-ADVANCED-MEDIA-SCHEDULING-AND-GRID-CONTROLS.md) | Advanced Media, Scheduling, and Grid Controls | `0.6.6` | Accepted / Normative | `tests/spec012_advanced_controls_tests.rs`, `tests/spec012_appkit_and_battery_parity_tests.rs`, `tests/appkit_v065_nib_parity_tests.rs` |

---

## 2. Roadmap & Capability Matrix

The roadmap coordinates modular capability additions across designated release milestones, ensuring mathematical verification and red test baselines precede production integrations:

### 2.1 Milestone Roadmap Overview

| Milestone | Target Release | Focus Area | Status | Normative Specifications |
| :--- | :--- | :--- | :--- | :--- |
| **Milestone 0.1.0** | `v0.1.0` (2026-11-05) | Core Engine, W3C DTCG Token Catalog & CASpring Physics | Shipped | SPEC-001, SPEC-003, SPEC-004 |
| **Milestone 0.2.0** | `v0.2.0` (2026-11-06) | Extended AppKit Controls, Layout Engines & 3-Tier Inspector | Shipped | SPEC-001 through SPEC-006 |
| **Milestone 0.3.0** | `v0.3.0` (2026-11-06) | Desktop Navigation, 24pt Canvas Footer & Spotlight Command Palette | Released | SPEC-007 (`0.3.0`) |
| **Milestone 0.4.0** | `v0.4.0` (2026-11-07) | State Integrity (`NSUndoManager`), Context Menus & Micro-Feedback | Released | SPEC-008 (`0.4.0`) |
| **Milestone 0.5.0** | `v0.5.0` (2026-11-08) | Studio & AVKit Media, Timeline and Publishing Controls | Released | SPEC-011 (`0.5.0`) |
| **Milestone 0.6.0** | `v0.6.0` (2026-11-10) | Advanced Media, Scheduling, and Grid Controls | Released | SPEC-012 (`0.6.0`) |

### 2.2 Functional Capability Matrix

| Capability Primitive | Module | Target Milestone | Spec ID | Verification Suite | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `MacBackgroundStyle` Contrast Engine | `src/theme/` | Milestone 0.3.0 | SPEC-002 | `tests/wcag_contrast_and_visuals_tests.rs` | Released |
| `MacMenuBar` / `NSMenu` / `NSMenuItem` | `src/navigation/` | Milestone 0.3.0 | SPEC-007 | `tests/menu_and_palette_tests.rs` | Released |
| Canvas Footer & Zoom HUD ($24\,\text{pt}$) | `src/navigation/` | Milestone 0.3.0 | SPEC-007 | `tests/menu_and_palette_tests.rs` | Released |
| Spotlight Command Palette (`Cmd+K`) | `src/navigation/` | Milestone 0.3.0 | SPEC-007 | `tests/menu_and_palette_tests.rs` | Released |
| Remote Differential Testing Harness | `tests/` & scripts | Milestone 0.3.0 | SPEC-009 | `tests/macbook_parity_differential_tests.rs` | Accepted / Normative |
| Interactive Gallery Controls & SF Symbols | `examples/` & `src/symbols` | Milestone 0.3.1 | SPEC-010 | `tests/micro_animations_and_hud_tests.rs`, `tests/symbol_tests.rs` | Released / Normative |
| `NSUndoManager` Ringbuffer ($C=50$) | `src/state/` | Milestone 0.4.0 | SPEC-008 | `tests/undo_and_feedback_tests.rs` | Released / Normative |
| Viewport-Clamped Context Menus | `src/feedback/` | Milestone 0.4.0 | SPEC-008 | `tests/undo_and_feedback_tests.rs` | Released / Normative |
| Animated Floating Toasts (`CASpring`, $2.5\,\text{s}$) | `src/feedback/` | Milestone 0.4.0 | SPEC-008 | `tests/undo_and_feedback_tests.rs` | Released / Normative |
| `MacProgressIndicator` (`Bar` & `Spinning`) | `src/controls/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacTableView` (`NSTableView` + Header) | `src/controls/table_view/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacTokenField` (`NSTokenField`) | `src/controls/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacLevelIndicator` (`NSLevelIndicator`) | `src/controls/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacTrimScrubber` (`AVPlayerView` Trim Bar) | `src/controls/trim_scrubber/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacPathControl` (`NSPathControl`) | `src/controls/` | Milestone 0.5.0 | SPEC-011 | `tests/studio_controls_v05_tests.rs` | Released / Normative |
| `MacPlayerView` (`AVPlayerView` Suite) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacDatePicker` (`NSDatePicker`) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacComboBox` (`NSComboBox`) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacTabView` (`NSTabView`) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacDisclosureGroup` (`NSButton` Disclosure) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacCollectionView` (`NSCollectionView`) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacBox` (`NSBox`) | `src/controls/` | Milestone 0.6.0 | SPEC-012 | `tests/spec012_advanced_controls_tests.rs` | Released / Normative |
| `MacRuleEditor` (`NSRuleEditor`) | `src/controls/` | Milestone 0.6.1 | SPEC-012 | `tests/spec012_appkit_and_battery_parity_tests.rs` | Released / Normative |

---

## 3. Crate & Spec Release Version Matrix

All core specifications in this repository maintain exact version parity with the public crate version in `Cargo.toml`. Future milestone specifications (SPEC-007, SPEC-008) advance through draft revisions ahead of formal crate version bumps.

| Crate Release | Spec Suite Version | Release Date | Key Invariants & Milestones |
| :--- | :--- | :--- | :--- |
| **v0.1.0** | `0.1.0` | 2026-11-05 | Baseline release: `MacButtonCell`, `MacInputCell`, `MacSegmentedControl`, `MacToggle`, `MacFixedView`, `MacFlexView`, `MacRow`, `CASpring` initial physics, W3C DTCG token catalog parser. |
| **v0.2.0** | `0.2.0` | 2026-11-06 | Previous milestone: Extended AppKit suite (`MacColorWell`, `MacOutlineView`, `MacPopUpButton`, `MacSearchField`, `MacSlider`, `MacStepper`, `MacThumbnailCell`), generic layout suite (`MacScrollView`, `MacStackView`, `MacSplitView`), `WindowControlsStyle` strategy (`AppleTrafficLights`, `WindowsCaptionButtons`, `NativeHost`), 3-tier inspector architecture (Compact HUD, Default NIB, Expanded Pro), modular specification decomposition under `specs/` (SPEC-001 through SPEC-006). Previews SPEC-007 and SPEC-008 navigation & feedback extensions. |
| **v0.3.0** | `0.3.0` | 2026-11-06 | Milestone 0.3.0 release: Promoted SPEC-007 to official release status. Full desktop navigation suite (`MacMenuBar`, `NSMenu`, `NSMenuItem` with cascading submenus and keyboard accelerators), deterministic 24pt canvas footer (`MacCanvasFooter` with breadcrumbs and zoom presets), Spotlight command palette modal HUD (`MacCommandPalette` with fuzzy subsequence scoring and Escape clearing mechanics), and formal `MacBackgroundStyle` dynamic contrast engine ($L \le 0.179 \implies$ Dark / On-Accent White text, $L > 0.179 \implies$ Light / Dark text) ensuring WCAG 2.1 AA $\ge 4.5:1$ contrast. |
| **v0.3.1** | `0.3.1` | 2026-11-06 | Milestone 0.3.1 release: Cross-platform parity (SPEC-010), Intel HD 5000 OpenGL fallback (`glow`), OS-adaptive window decorations (`decorations = true` on Windows, `false` on macOS), full bidirectional state binding (Zero Dead Controls: Preset popup, live scale slider, corner stepper, units formatter, CASpring touch pulse), rich vector iconography via `MacSymbol` (Cupertino SF Symbols), and cross-platform Spotlight Command Palette shortcut handling (`⌘K` / `Ctrl+K`). |
| **v0.4.0** | `0.4.0` | 2026-11-07 | Milestone 0.4.0 release: Promoted SPEC-008 (State Integrity & Feedback) to official release status. Bounded ringbuffer transaction stack (`NSUndoManager<T>`, capacity 50) with cyclic state preservation and full rollback; viewport-boundary clamped context menus (`MacContextMenu`, `clamp_menu_rect`) with click-outside dismissal; animated micro-feedback toast notifications (`MacToast`, `MacToastManager`) powered by `CASpring` entry/exit physics with 2.5s auto-dismiss timer; wired into `macview_gallery` interactive canvas card with global shortcuts (`⌘Z` / `Ctrl+Z`, `⌘⇧Z` / `Ctrl+Shift+Z`) and context actions (Reset Transform, Center Card, Fill Width, Toggle Proportions, Copy Style). |
| **v0.4.1** | `0.4.1` | 2026-11-08 | Milestone 0.4.1 consolidation & hardening release: Zero hardcoding across all UI modules via expanded W3C DTCG tokens; unmultiplied RGBA DTCG parsing; transactional `MacInputCell` edit buffer with `Escape` rollback and domino-free `Tab` chain; first-class `MacToolbar` (`SPEC-005`) and `MacFloatingInfoBox` (`SPEC-006 §2.2`); foreground click/scroll isolation across popovers, panels, and dropdowns; removed empty `MacFlexView` stub. |
| **v0.5.0** | `0.5.0` | 2026-11-08 | Milestone 0.5.0 release: Promoted SPEC-011 (Studio & AVKit Controls) to official release status. Six domain-agnostic studio, media, timeline, and publishing controls: `MacProgressIndicator` (`Bar` determinate/indeterminate + `Spinning` 12-spoke fin indicator across `Regular`/`Small`/`Mini`), `MacTableView` (multi-column `NSTableView` + `NSTableHeaderView` with sort chevrons, alternating rows, single/range/multi selection, and consumed arrow-key navigation), `MacTokenField` (flow-wrapping `NSTokenField` pill editor with comma/Enter commit, Backspace/`×` removal, and character counter), `MacLevelIndicator` (continuous/discrete horizontal/vertical audio & signal meter with warning/critical thresholds and peak hold), `MacTrimScrubber` (QuickTime/AVKit waveform timeline with named zones, In/Out trim handles, and `MM:SS.d` playhead badge), and `MacPathControl` (clickable `NSPathControl` breadcrumb bar with `Choose...` button); plus unmultiplied RGBA DTCG token parsing. |
| **v0.5.1** | `0.5.1` | 2026-11-08 | Milestone 0.5.1 hardening release: Automated Architecture Fitness Guards (`tests/architecture_guard_tests.rs`), cross-platform GPU hardening (`glow`), consolidated single-pass `Arc<Galley>` reuse and `egui::Memory` lookups, and runtime hygiene/invariants test suites. |
| **v0.5.2** | `0.5.2` | 2026-11-10 | Milestone 0.5.2 release: Added `MacCircularSlider` and `MacRatingIndicator` primitives, live AVKit audio/video synthesis with 30 FPS video canvas, authentic Finder browsing with pinned parent directory (`..`) and Windows drive root chain (`C:\`, `D:\`), interactive Rust snippet generator, WCAG AA contrast refinement, zero-idle battery efficiency, and formal specification of `SPEC-012` for Milestone 0.6.0. |
| **v0.6.0** | `0.6.0` | 2026-11-10 | Milestone 0.6.0 release: Promoted SPEC-012 (Advanced Media, Scheduling, and Grid Controls) to official release status. Seven advanced AppKit/AVKit controls: `MacPlayerView` (QuickTime floating HUD pod, inline transport, minimal circular transport), `MacDatePicker` (text-field with stepper, clock face, calendar matrix), `MacComboBox` (editable input with dropdown trigger and viewport-clamped popover), `MacTabView` (bezel and borderless segmented tab container with flipped coordinates), `MacDisclosureGroup` & `MacDisclosureButton` (hierarchical collapsible toggle with CASpring rotation and pill button), `MacCollectionView` (2D grid engine with 2D viewport culling and spatial keyboard navigation), and `MacBox` (container framing with title badge and content insets); plus 6-state interactive gallery showcase (`macview_gallery`). |
| **v0.6.1** | `0.6.1` | 2026-11-10 | Milestone 0.6.1 release: Added `MacRuleEditor` (`REQ-012-07`), full AVKit transport/audio/speed controls in `MacPlayerView`, zero idle heap allocations via `MacTheme::resolve_or_default`, stationary-cursor HUD auto-hide, $O(1)$ collection view row-range culling, and `spec012_appkit_and_battery_parity_tests.rs`. |
| **v0.6.2** | `0.6.2` | 2026-11-10 | Milestone 0.6.2 release: Added standalone `eframe` quickstart app, Apple HIG "When to Use Which Control" decision tables, Troubleshooting & FAQ section, IDE hover `/// # Example` rustdoc blocks across all `v0.5`–`v0.6` controls, and Mermaid architecture diagrams in `ARCHITECTURE.md`. |
| **v0.6.3** | `0.6.3` | 2026-11-10 | Milestone 0.6.3 release: Converted 62 `README.md` API reference links to rustdoc intra-doc links (`[`Type`]`) for `docs.rs`, added 9 `/// # Example` rustdoc blocks on Navigation, Surfaces, Feedback, State, and Symbol types, fixed `ARCHITECTURE.md` and `CHANGELOG.md` links, and added `README.md` intra-doc link architecture guard. |
| **v0.6.4** | `0.6.4` | 2026-11-10 | Milestone 0.6.4 release: Repository-wide 100% English comment/prose/package verification guard (`check_vault_leakage_and_dutch_in_text`), removal of legacy `AUDIT-REFACTORING-REPORT.md` from version control, standardized `Zero Dead Code Guard` terminology, and portable `%~dp0` resolution in `start_gallery.cmd`. |
| **v0.6.5** | `0.6.5` | 2026-11-11 | Milestone 0.6.5 release: Authentic AppKit NIB parity (`MacCheckbox`, `MacLabel`, `MacHelpButton`, `MacInlineActionButton`), universal optical letter-spacing tracking across all text-rendering controls, cross-platform baseline anchoring (`first_baseline_offset_from_top - ascender`), and bundled open-source `Inter` font (`register_macview_fonts`). |
| **v0.6.6** | `0.6.6` | 2026-11-11 | Milestone 0.6.6 release: Physical MacBook Air (`macOS Sonoma 14.8.9`) `NSTextFieldSquareBezel` corner-radius parity (`2.0pt` for `Regular`/`Small`/`CompactInfobox`, `1.5pt` for `Mini`), WCAG 2.1 AA Aqua `secondary_label` contrast hardening (`#5A5A5F`, `5.48:1` on `MacBox`), `MacHelpButton` (`12pt`/`10pt`/`9pt` `Inter-SemiBold`), `MacPopUpButton::accent_badge(bool)`, and zero-allocation `tracking_for_font_size` & `semibold_font_id` lookups. |

---

## 4. Normative Industry Quality Standards

Every specification and code implementation SHALL adhere strictly to:
1. **ISO/IEC/IEEE 42010:2011:** Strict separation between Model, View, and Controller viewpoints.
2. **ISO/IEC/IEEE 29119-3:2021:** Formal test assertions and red COTS baseline requirements.
3. **Apple macOS Human Interface Guidelines (HIG):** Exact metrics, 1px hairlines, outside focus rings, and optical typography tracking.
4. **W3C Design Tokens Community Group (DTCG):** 100% data-driven token catalogs (`tokens/design_tokens.json`). Zero magic numbers or hardcoded colors.
5. **W3C WCAG 2.1 §1.4.3 (Level AA):** Relative luminance contrast $\ge 4.5:1$ across Aqua and DarkAqua.
6. **Automated Architecture Fitness Standards:** Modularity hard ceilings (<400 eSLOC, <599 physical lines), zero domain leakage, and zero token arithmetic cheating.
