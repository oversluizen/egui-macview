# SPEC-008: State Integrity, Context Menus & Micro-Feedback Architecture

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-008 |
| **Document Title** | State Integrity, Context Menus & Micro-Feedback Architecture |
| **Document Version** | `0.4.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.4.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/undo_and_feedback_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | Architecture Team | Baseline Initial Release: State Management Foundation |
| `0.2.0` | 2026-11-06 | Architecture Team | Prototyping Milestone: NSUndoManager Bounded Ringbuffer, Clamped Context Menus, and Animated Toast Notifications |
| `0.4.0` | 2026-11-07 | Architecture Team | Milestone 0.4.0 Release: Transactional state integrity, boundary-clamped context menus, and CASpring micro-feedback |

---

## 1. Scope & System Architecture

State integrity and responsive visual feedback are critical hallmarks of high-assurance macOS applications. Users rely unconditionally on non-destructive undo/redo stacks, predictable contextual interactions on cards and tree nodes, and transient status banners.

SPEC-008 defines three core state and feedback mechanisms:
1. **`NSUndoManager`:** Bounded ringbuffer transaction stack managing snapshot history with capacity limits, zero memory leakage, and property-verified cyclic restoration.
2. **Context Menu (`MacContextMenu`):** Secondary click (right-click) context popover with boundary clamping guaranteeing zero clipping across viewport edges.
3. **Animated Toast Notifications (`MacToast`):** Floating pill banner with `CASpring` entry/exit physics and $2.5\,\text{s}$ auto-dismiss timer.

---

## 2. `NSUndoManager<T>` Specification

### 2.1 Bounded Ringbuffer State Architecture
`NSUndoManager` maintains two bounded deques: `undo_stack` and `redo_stack`.

```rust
use std::collections::VecDeque;

pub const DEFAULT_UNDO_CAPACITY: usize = 50;

#[derive(Debug, Clone)]
pub struct NSUndoManager<T> {
    undo_stack: VecDeque<T>,
    redo_stack: VecDeque<T>,
    capacity: usize,
}
```

### 2.2 Formal State Invariants
1. **Capacity Boundary:**
   $$\forall t: \text{len}(\text{undo\_stack}) \le C \quad \land \quad \text{len}(\text{redo\_stack}) \le C$$
   When an operation exceeds capacity $C$, the oldest snapshot is evicted via `pop_front()`.
2. **Redo Stack Invalidation:**
   Pushing a new transactional snapshot onto the undo stack SHALL unconditionally clear the redo stack:
   $$\text{push\_snapshot}(s) \implies \text{redo\_stack} = \emptyset$$
3. **Cyclic State Preservation (Property Oracle):**
   For any initial state $S_0$ and series of mutations producing $S_1, S_2, \dots, S_n$:
   Undoing $k$ times restores state $S_{n-k}$ identically.
   Subsequent redoing $k$ times restores state $S_n$ identically.

### 2.3 Operations API
- `NSUndoManager::new()`: Initializes with default capacity ($50$).
- `NSUndoManager::with_capacity(capacity)`: Custom capacity limit.
- `push_snapshot(state: T)`: Records current state snapshot; drops oldest if over capacity; clears redo stack.
- `undo(current_state: &mut T) -> bool`: Pushes `*current_state` to redo; pops previous from undo into `*current_state`. Returns `false` if `undo_stack.is_empty()`.
- `redo(current_state: &mut T) -> bool`: Pushes `*current_state` to undo; pops next from redo into `*current_state`. Returns `false` if `redo_stack.is_empty()`.
- `can_undo(&self) -> bool`
- `can_redo(&self) -> bool`
- `clear(&mut self)`: Clears both stacks.

---

## 3. Context Menu (`MacContextMenu`) Specification

### 3.1 Invocation & Boundary Clamping
A contextual menu is invoked on secondary click (right-click). To ensure menu items are never clipped offscreen, the spawn position is clamped against viewport bounds:

$$\begin{aligned}
x_{\text{clamped}} &= \min(P_x, R_{\text{right}} - W_{\text{menu}}) \\
y_{\text{clamped}} &= \min(P_y, R_{\text{bottom}} - H_{\text{menu}}) \\
x_{\text{final}} &= \max(x_{\text{clamped}}, R_{\text{left}}) \\
y_{\text{final}} &= \max(y_{\text{clamped}}, R_{\text{top}})
\end{aligned}$$

Where:
- $P(x, y)$ is the click pointer coordinate.
- $W_{\text{menu}}, H_{\text{menu}}$ are the menu dimensions.
- $R$ is the enclosing viewport / window rectangle.

### 3.2 Dismissal & Selection
- Left-clicking any enabled item executes the item action and dismisses the context menu.
- Clicking outside the context menu bounds or pressing `Escape` dismisses the context menu without action.

---

## 4. Animated Toast Notifications (`MacToast`)

### 4.1 Form Factor & Presentation
Toasts appear as a floating pill banner centered horizontally near the top or bottom of the application window:
- **Corner Radius:** Pill squircle (`corner_radius = height * 0.5`).
- **Padding:** $12.0\,\text{pt}$ horizontal, $6.0\,\text{pt}$ vertical.
- **Typography:** $12.0\,\text{pt}$ medium weight label text.
- **Elevation:** Drop shadow with blur $8.0\,\text{pt}$ and $20\%$ black opacity.

### 4.2 CoreAnimation Physics & Lifecycle Timing
Toast animations are driven by `CASpring`:
- **Spring Parameterization:** `response = 0.35s`, `damping_ratio = 0.85`.
- **Settling Bound:** Oscillator settles to $< 0.1\%$ resting deviation in $\approx 0.33\,\text{s}$.
- **Lifecycle Timeline:**
  1. **Entry Phase ($t \in [0.0, 0.35\,\text{s}]$):** Translates into viewport using `CASpring::evaluate(t)`.
  2. **Display Phase ($t \in [0.35\,\text{s}, 2.50\,\text{s}]$):** Fully visible and interactive.
  3. **Auto-Dismiss Trigger ($t = 2.50\,\text{s}$):** Auto-dismiss timer fires after $2.5\,\text{s}$.
  4. **Exit Phase ($t > 2.50\,\text{s}$):** Animates out via spring or fade out and is pruned from the toast queue.

---

## 5. ISO 29119-3 Test Traceability Matrix

| Requirement ID | Verification Target | Test Method | Test Oracle / Acceptance Criterion |
| :--- | :--- | :--- | :--- |
| `REQ-008-01` | `NSUndoManager` cyclic preservation | `proptest` | Arbitrary sequences of mutations, undos, and redos strictly restore previous states. |
| `REQ-008-02` | `NSUndoManager` bounded capacity | `proptest` | Stacks never exceed `capacity`; memory remains bounded with zero leaks. |
| `REQ-008-03` | Context Menu Boundary Clamping | Unit Test | Clamped menu rectangle is completely contained within container bounds. |
| `REQ-008-04` | Toast Notification Auto-Dismiss | Headless `egui` | Toast is active for $t < 2.5\,\text{s}$ and marked dismissed for $t \ge 2.5\,\text{s}$. |
| `REQ-008-05` | Toast Spring Settling Bounds | Analytical Unit | `CASpring` entry settling occurs within $0.5\,\text{s}$ HIG budget. |
