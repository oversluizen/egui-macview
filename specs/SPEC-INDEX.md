# SPEC-INDEX: `egui-macview` Specification Suite Index & Version Matrix

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-INDEX |
| **Document Title** | Specification Suite Index & Version Matrix |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: Monolithic Architecture Specification |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Decomposed Modular Specification Suite Synchronized with Crate v0.2.0 |

---

## 1. Specification Suite Structure

The `egui-macview` specification suite provides the formal normative requirements, mathematical foundations, MVC abstractions, and test oracles for the entire crate. Every specification in this directory runs strictly in lockstep with the crate release version (`0.2.0`).

| Spec ID | Document Title | Version | Status | Primary Test Suite |
| :--- | :--- | :--- | :--- | :--- |
| [**SPEC-001**](SPEC-001-DESIGN-TOKENS-AND-WCAG.md) | Design Tokens, Optical Tracking & WCAG AA Contrast | `0.2.0` | Accepted | `tests/wcag_contrast_and_visuals_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs` |
| [**SPEC-002**](SPEC-002-CONTROLS-AND-MICRO-ANIMATIONS.md) | Authentic AppKit Controls & Micro-Animation Oracles | `0.2.0` | Accepted | `tests/appkit_primitives_tests.rs`, `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-003**](SPEC-003-LAYOUT-ENGINES.md) | Deterministic Layout Engines & Subpixel Snap Rasterization | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/appkit_primitives_tests.rs` |
| [**SPEC-004**](SPEC-004-CORE-ANIMATION-PHYSICS.md) | CoreAnimation Spring Physics Runtime (`CASpring`) | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/macbook_parity_differential_tests.rs` |
| [**SPEC-005**](SPEC-005-WINDOW-FRAMING-AND-CONTROLS.md) | Window Framing, Drag Handling & Configurable Caption Controls | `0.2.0` | Accepted | `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-006**](SPEC-006-INSPECTOR-ARCHITECTURE.md) | Three-Tier Inspector Architecture & Floating Surfaces | `0.2.0` | Accepted | `tests/cots_red_baseline_tests.rs`, `tests/micro_animations_and_hud_tests.rs` |
| [**SPEC-007**](SPEC-007-DESKTOP-NAVIGATION-AND-PALETTE.md) | Desktop Navigation, Canvas Footer & Spotlight Command Palette | `0.2.0` | Accepted | `tests/menu_and_palette_tests.rs` |
| [**SPEC-008**](SPEC-008-STATE-INTEGRITY-AND-FEEDBACK.md) | State Integrity, Context Menus & Micro-Feedback Architecture | `0.2.0` | Accepted | `tests/undo_and_feedback_tests.rs` |

---

## 2. Crate & Spec Release Version Matrix

All specifications in this repository maintain exact 1-to-1 version parity with the public crate version in `Cargo.toml`.

| Crate Release | Spec Suite Version | Release Date | Key Invariants & Milestones |
| :--- | :--- | :--- | :--- |
| **v0.1.0** | `0.1.0` | 2026-11-05 | Baseline release: `MacButtonCell`, `MacInputCell`, `MacSegmentedControl`, `MacToggle`, `MacFixedView`, `MacFlexView`, `MacRow`, `CASpring` initial physics, W3C DTCG token catalog parser. |
| **v0.2.0** | `0.2.0` | 2026-11-06 | Current milestone: Extended AppKit suite (`MacColorWell`, `MacOutlineView`, `MacPopUpButton`, `MacSearchField`, `MacSlider`, `MacStepper`, `MacThumbnailCell`), generic layout suite (`MacScrollView`, `MacStackView`, `MacSplitView`), desktop navigation (`NSMenu`, `NSMenuItem`, `MacMenuBar`, `MacCanvasFooter`, `MacCommandPalette`), state integrity and feedback (`NSUndoManager`, `MacContextMenu`, `MacToast`), `WindowControlsStyle` strategy (`AppleTrafficLights`, `WindowsCaptionButtons`, `NativeHost`), 3-tier inspector architecture (Compact HUD, Default NIB, Expanded Pro), modular specification decomposition under `specs/` (SPEC-001 through SPEC-008), full ISO 29119-4 property-based test harness, architecture guardrails. |

---

## 3. Normative Industry Quality Standards

Every specification and code implementation SHALL adhere strictly to:
1. **ISO/IEC/IEEE 42010:2011:** Strict separation between Model, View, and Controller viewpoints.
2. **ISO/IEC/IEEE 29119-3:2021:** Formal test assertions and red COTS baseline requirements.
3. **Apple macOS Human Interface Guidelines (HIG):** Exact metrics, 1px hairlines, outside focus rings, and optical typography tracking.
4. **W3C Design Tokens Community Group (DTCG):** 100% data-driven token catalogs (`tokens/design_tokens.json`). Zero magic numbers or hardcoded colors.
5. **W3C WCAG 2.1 §1.4.3 (Level AA):** Relative luminance contrast $\ge 4.5:1$ across Aqua and DarkAqua.
6. **GL-1015 Software Engineering Standards:** Modularity hard ceilings (<400 eSLOC, <599 physical lines), zero domain leakage, and zero token arithmetic cheating.
