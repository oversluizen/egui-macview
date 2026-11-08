# SPEC-INDEX: `egui-macview` Specification Suite Index & Version Matrix

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-INDEX |
| **Document Title** | Specification Suite Index & Version Matrix |
| **Document Version** | `0.5.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.5.0 |
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

---

## 1. Specification Suite Structure

The `egui-macview` specification suite provides the formal normative requirements, mathematical foundations, MVC abstractions, and test oracles for the entire crate. Every specification in this directory runs strictly in lockstep with the crate release version (`0.5.0`).

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

---

## 4. Normative Industry Quality Standards

Every specification and code implementation SHALL adhere strictly to:
1. **ISO/IEC/IEEE 42010:2011:** Strict separation between Model, View, and Controller viewpoints.
2. **ISO/IEC/IEEE 29119-3:2021:** Formal test assertions and red COTS baseline requirements.
3. **Apple macOS Human Interface Guidelines (HIG):** Exact metrics, 1px hairlines, outside focus rings, and optical typography tracking.
4. **W3C Design Tokens Community Group (DTCG):** 100% data-driven token catalogs (`tokens/design_tokens.json`). Zero magic numbers or hardcoded colors.
5. **W3C WCAG 2.1 §1.4.3 (Level AA):** Relative luminance contrast $\ge 4.5:1$ across Aqua and DarkAqua.
6. **GL-1015 Software Engineering Standards:** Modularity hard ceilings (<400 eSLOC, <599 physical lines), zero domain leakage, and zero token arithmetic cheating.
