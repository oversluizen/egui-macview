# SPEC-003: Deterministic Layout Engines & Subpixel Snap Rasterization

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-003 |
| **Document Title** | Deterministic Layout Engines & Subpixel Snap Rasterization |
| **Document Version** | `0.2.0` |
| **Status** | Accepted / Normative |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/cots_red_baseline_tests.rs`, `tests/appkit_primitives_tests.rs`, `tests/high_assurance_invariants_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: MacFixedView, MacFlexView, MacRow & Subpixel Snap Algorithm |
| `0.2.0` | 2026-11-06 | Architecture Team | Current Milestone: Generic Container Suite (MacScrollView, MacStackView, MacSplitView) with Overlay Scrollbars |

---

## 1. Scope & Mathematical Foundations

This specification defines the deterministic layout algorithms and rasterization criteria eliminating visual clipping, fractional blur, and edge fringing across all display scaling configurations ($1.0\times$, $1.25\times$, $1.5\times$, $1.75\times$, $2.0\times$).

### 1.1 Subpixel Snap Criterion
To eliminate fractional blur across non-integer display scaling factors, logical coordinates in points (`pt`) MUST snap to physical device pixels (`px`) according to:

$$x_{\text{pixel}} = \frac{\lfloor x_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$

$$y_{\text{pixel}} = \frac{\lfloor y_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$

Where $s$ is the display scale factor (`pixels_per_point`). Every hairline stroke ($1.0\,\text{pt}$), separator line, and container boundary is snapped before rendering.

---

## 2. Layout Container Suite

### 2.1 `MacFixedView` (Absolute NIB Placement)
- **Role:** Implements explicit, retained-geometry bounding boxes identical to macOS AppKit NIB/XIB files.
- **Specification:**
  - Container defined by an exact logical `Rect` in points.
  - Child elements positioned via local offset coordinates $(x, y)$ and explicit size $(w, h)$.
  - Zero internal automatic padding or margin injection: child bounds are preserved to the exact point.
  - Zero-clipping invariant: if $\sum w_i + \text{gaps} \le W_{\text{container}}$, clipping is provably impossible.

### 2.2 `MacFlexView` (Springs & Struts Layout)
- **Role:** Implements classic macOS AutoLayout springs and struts mechanics.
- **Specification:**
  - Edges can be fixed (struts) or flexible (springs).
  - Width and height can expand proportionally with container resizing while maintaining anchored margins.

### 2.3 `MacRow` (Auto-Budgeting Horizontal Layout)
- **Role:** Distributes horizontal space among an ordered series of control cells.
- **Specification:**
  - Accepts a fixed or bounded total width budget $W$.
  - Allocates fixed-width slots (e.g. labels, steppers, icons) first.
  - Distributes remaining space $\Delta W = W - \sum W_{\text{fixed}} - \sum \text{spacing}$ equally or proportionally across flexible cells.

### 2.4 `MacSplitView` (`NSSplitView` Equivalent)
- **Role:** Multi-pane adjustable layout separated by draggable dividers.
- **Specification:**
  - **Orientation:** Horizontal (side-by-side panes) or Vertical (stacked panes).
  - **Hairline Divider:** Drawn at exact $1.0\,\text{pt}$ thickness (`split_view.divider_thickness`), subpixel snapped.
  - **Hit Target:** Symmetrical $6.0\,\text{pt}$ interactive drag zone (`split_view.hit_thickness`) centered on the divider line.
  - **Constraints:** Minimum pane size enforcement (`min_pane_size`, default $50.0\,\text{pt}$) preventing pane collapse beyond usable limits.
  - **Visual Drag Handle:** Optional centered drag pill ($24.0 \times 3.0\,\text{pt}$) rendered in secondary label / separator tint.
  - **Interaction:** Live mouse tracking with cursor update (`ResizeHorizontal` / `ResizeVertical`).

### 2.5 `MacScrollView` (`NSScrollView` Equivalent)
- **Role:** High-performance, viewport-clipped scrollable container featuring authentic macOS floating overlay scrollbars.
- **Specification:**
  - **Zero-Gutter Content Reservation:** Unlike standard X11/Windows scroll views that permanently reserve a 16px gutter, content occupies 100% of the viewport width and height. Floating scrollbars overlay the content surface.
  - **Dynamic Overlay Geometry:**
    - Scrollbar thickness: $7.0\,\text{pt}$ (`scroll_view.scrollbar_thickness`).
    - Minimum thumb length: $20.0\,\text{pt}$ (`scroll_view.thumb_min_length`).
    - Pill corner radius: $3.5\,\text{pt}$ (`scroll_view.corner_radius`).
    - Viewport edge margin inset: $3.0\,\text{pt}$ (`scroll_view.edge_inset`).
  - **Dual-Overlay Corner Clearance:** When both horizontal and vertical overlay scrollbars are active, the vertical track stops above the horizontal scroller and the horizontal track stops left of the vertical scroller by `edge_inset + scrollbar_thickness + edge_inset`, provably preventing bottom-right intersection.
  - **Auto-Fade Inactivity Timer:** Scrollbars render at 100% opacity during pointer interaction, dragging, or wheel scrolling, and smoothly fade to 0.0 opacity after an inactivity duration of $0.8\,\text{s}$ (`scroll_view.fade_timeout`). Moving the pointer into the track activation zone smoothly wakes the scrollbar.
  - **Interaction & Smooth Clipping:** Child elements outside the viewport bounding box are hardware clipped using `ui.painter().with_clip_rect()`.

### 2.6 `MacStackView` (`NSStackView` Equivalent)
- **Role:** Dynamic linear auto-stacking container for ordered component arrangements along horizontal or vertical axes.
- **Specification:**
  - **Orientation:** Vertical (column stack, default) or Horizontal (row stack).
  - **Token-Driven Spacing & Insets:**
    - Inter-item spacing: $8.0\,\text{pt}$ (`stack_view.default_spacing`), dynamically customizable per instance.
    - Outer edge padding: $8.0\,\text{pt}$ (`stack_view.edge_padding`).
    - Section header typography: $11.0\,\text{pt}$ (`stack_view.section_header_font_size`).
  - **Section Grouping & Hairline Dividers:**
    - Supports explicit section boundaries with hairline separator lines of $1.0\,\text{pt}$ thickness (`stack_view.separator_thickness`).
    - Separator strokes span the entire cross-axis dimension and resolve dynamically from `MacTheme::separator()` color tokens.
  - **Deterministic Layout Allocation:** Explicit container width and height constraints are strictly enforced via `ui.allocate_rect()`, propagating bounding constraints and eliminating cumulative layout drift.
