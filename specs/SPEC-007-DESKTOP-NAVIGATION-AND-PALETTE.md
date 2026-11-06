# SPEC-007: Desktop Navigation, Canvas Footer & Spotlight Command Palette

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-007 |
| **Document Title** | Desktop Navigation, Canvas Footer & Spotlight Command Palette |
| **Document Version** | `0.3.0-draft` |
| **Status** | Draft / Normative |
| **Target Milestone** | Milestone 0.3.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/menu_and_palette_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: Baseline Navigation Concepts |
| `0.2.0` | 2026-11-06 | Architecture Team | Prototyping Milestone: Desktop Navigation (NSMenu/NSMenuItem), 24pt Canvas Footer Layout, and Spotlight Command Palette |
| `0.3.0-draft` | 2026-11-06 | Architecture Team | Target Milestone 0.3.0: Data-driven menu bar, cascading submenu popovers, 24pt footer, and Spotlight Command Palette with fuzzy search |

---

## 1. Scope & System Architecture

Professional macOS desktop applications require structured top-level navigation, immediate visual context readouts, and rapid keyboard-first access to all available commands. 

SPEC-007 defines three foundational desktop navigation primitives for `egui-macview`:
1. **`NSMenu` / `NSMenuItem` & `MacMenuBar`:** Data-driven top application menu bar with active states, keyboard shortcut accelerators, and cascading submenu popovers.
2. **Canvas Footer & Zoom HUD:** Deterministic $24.0\,\text{pt}$ layout container via `MacRow` providing breadcrumb hierarchy paths, cursor coordinates, and zoom presets (`50% | 100% | 200%`).
3. **Spotlight Command Palette (`Cmd+K` / `Ctrl+K`):** Floating $480.0 \times 280.0\,\text{pt}$ modal HUD with real-time fuzzy query filtering, categorized commands, and full keyboard navigation.

---

## 2. `NSMenu` / `NSMenuItem` & Menu Bar Architecture

### 2.1 Data-Driven Menu Model

Menu trees SHALL be represented as recursive, pure data structures decoupled from immediate-mode rendering frames:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MacKeyboardShortcut {
    pub key: egui::Key,
    pub command: bool,
    pub shift: bool,
    pub alt: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacMenuItemKind {
    Action {
        id: String,
        shortcut: Option<MacKeyboardShortcut>,
    },
    Submenu {
        items: Vec<NSMenuItem>,
    },
    Separator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NSMenuItem {
    pub title: String,
    pub kind: MacMenuItemKind,
    pub enabled: bool,
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NSMenu {
    pub title: String,
    pub items: Vec<NSMenuItem>,
}
```

### 2.2 Standard Root Menu Bar Structure
A desktop application menu bar SHALL provide five standard top-level menus:
1. **File:** New, Open (`Ctrl+O`), Save (`Ctrl+S`), Export, Close.
2. **Edit:** Undo (`Ctrl+Z`), Redo (`Ctrl+Shift+Z` / `Ctrl+Y`), Cut (`Ctrl+X`), Copy (`Ctrl+C`), Paste (`Ctrl+V`), Select All (`Ctrl+A`).
3. **View:** Zoom In (`Ctrl++`), Zoom Out (`Ctrl+-`), Actual Size (`Ctrl+0`), Toggle Inspector, Fullscreen.
4. **Window:** Minimize, Zoom, Bring All to Front.
5. **Help:** Documentation, Keyboard Shortcuts (`Cmd+K` / `Ctrl+K`), About.

### 2.3 Keyboard Navigation Oracles
When a menu is opened:
- **Down Arrow (`↓`):** Highlights next enabled menu item (wrapping or clamping). Separator items SHALL be bypassed.
- **Up Arrow (`↑`):** Highlights previous enabled menu item.
- **Right Arrow (`→`):** Expands child submenu popover if current item is `MacMenuItemKind::Submenu`.
- **Left Arrow (`←`):** Collapses child submenu popover and returns highlight to parent item. If in a root menu, switches to the previous root menu.
- **Enter / Space:** Executes the highlighted action and closes all open menus.
- **Escape (`Esc`):** Closes the current menu level or the entire menu hierarchy immediately.

---

## 3. Canvas Footer & Zoom HUD Specification

### 3.1 Deterministic $24.0\,\text{pt}$ Height Budget
The canvas footer occupies a fixed horizontal strip across the bottom of the workspace. Its vertical layout budget is strictly $24.0\,\text{pt}$ (`footer.height = 24.0pt`).

### 3.2 Layout Allocation via `MacRow`
All footer elements SHALL be placed within a `MacRow` container to enforce zero clipping and subpixel alignment:

$$\text{MacRow}::\text{new}(y_{\text{offset}}, 24.0\,\text{pt}, W_{\text{container}}, \text{spacing})$$

The footer partition consists of three zones:
1. **Leading Zone (Breadcrumbs):**
   - Hierarchical trail (e.g. `Document > Artboard 1 > Group > VectorCard`).
   - Sizing: Proportional to text width, truncation with ellipsis (`...`) if container shrinks.
2. **Center Zone (Coordinate Readout):**
   - Precise cursor coordinates relative to canvas origin: e.g. `X: 420.5 pt   Y: 180.0 pt`.
   - Typography: Monospace tabular numerals (`egui::FontId::monospace(11.0)`).
3. **Trailing Zone (Zoom HUD):**
   - Zoom presets: `50% | 100% | 200%` rendered as segmented control or compact button cluster.
   - Stepper or percentage readout for arbitrary zoom factors ($10\% \le z \le 3200\%$).

---

## 4. Spotlight Command Palette (`Cmd+K` / `Ctrl+K`)

### 4.1 Surface Metrics & Geometry
The command palette is an authentic floating AppKit HUD surface:
- **Width:** $480.0\,\text{pt}$ (`command_palette.width = 480.0pt`).
- **Height:** $280.0\,\text{pt}$ (`command_palette.height = 280.0pt`).
- **Corner Radius:** $10.0\,\text{pt}$ continuous curvature squircle (`command_palette.corner_radius = 10.0pt`).
- **Elevation:** Modal backdrop with semi-transparent overlay (`rgba(0, 0, 0, 0.40)`).
- **Search Header:** $36.0\,\text{pt}$ height with magnifying glass icon and placeholder `"Type a command or search..."`.
- **Result Row Height:** $28.0\,\text{pt}$ (`command_palette.item_height = 28.0pt`).

### 4.2 Keyboard Navigation & Fuzzy Query Filtering
- **Activation:** `Cmd+K` (macOS) or `Ctrl+K` (Windows/Linux) global shortcut.
- **Search Query:** Incremental substring and fuzzy character matching across `title` and `category`.
- **Selected Index:** Default index is $0$.
  - **Down Arrow (`↓`):** Increments selected index, clamped to `results.len() - 1`.
  - **Up Arrow (`↑`):** Decrements selected index, clamped to $0$.
  - **Enter:** Fires the selected action trigger and closes the palette.
  - **Escape:** Clears the query or dismisses the palette modal.

---

## 5. ISO 29119-3 Test Traceability Matrix

| Requirement ID | Verification Target | Test Method | Test Oracle / Acceptance Criterion |
| :--- | :--- | :--- | :--- |
| `REQ-007-01` | `NSMenu` data structure | Unit Test | Pure data models initialize, clone, and serialize correctly. |
| `REQ-007-02` | Menu keyboard navigation | Headless `egui` | Arrow keys navigate indices; Enter fires action; Escape closes menu. |
| `REQ-007-03` | Canvas Footer $24\,\text{pt}$ Budget | Headless `egui` | `MacRow` bounds remain strictly $\le 24.0\,\text{pt}$ height without vertical overflow. |
| `REQ-007-04` | Command Palette $480 \times 280\,\text{pt}$ | Headless `egui` | Rect matches $480 \times 280\,\text{pt}$; items render within scroll bounds. |
| `REQ-007-05` | Command Palette Navigation | Headless `egui` | Query filtering updates list; Arrow keys shift selection; Enter triggers callback. |
