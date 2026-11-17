# SPEC-014: AppKit "Liquid Glass" 2-Layer Context Thumb Dial (`MacContextThumbDial`) & Top-Down Free-Spin Edge Thumbwheel (`MacEdgeThumbwheel`)

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-014 |
| **Document Title** | AppKit "Liquid Glass" 2-Layer Context Thumb Dial (`MacContextThumbDial`) & Top-Down Free-Spin Edge Thumbwheel (`MacEdgeThumbwheel`) |
| **Document Version** | `0.8.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.8.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/spec014_context_thumb_dial_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, Apple macOS / iPadOS HIG, `pie-menu.com` Liquid Glass Aesthetics, Buxton & Kurtenbach Radial Marking Menus, Guiard Kinematic Chain Theory, W3C DTCG, SemVer 2.0.0 |

---

## 1. Architectural Rationale & Two-Component Mobile/Tablet Ergonomics

Combining compass modifier slices, 360° point nudging, and a concentric rotary wheel into a single 4-ring dial produces a footprint that is too large for a smartphone screen. To achieve compact mobile/tablet ergonomics with authentic macOS AppKit / `pie-menu.com` **Liquid Glass** aesthetics (in both `DarkAqua` and `Aqua` Light Mode), `SPEC-014` splits the bimanual controller into **two compact, complementary Liquid Glass widgets**:

1. **Component A — `MacContextThumbDial` (Compact 2-Layer Liquid Glass Thumb Puck + Curved `↶ Undo | ↷ Redo` Crescents):**
   - **Layer 1 (`MacThumbDialLayer::CompassModifiers`):** Six rounded frosted-glass radial cushions (`North`, `NorthEast`, `East`, `SouthEast`, `South`, `West`) displaying live tool-specific action labels (`1:1`, `1:1+⊙Center`, `⊙ Center`, `⇄ Shear`, `↝ Slide`, `✥ Move`) with *Tap-to-Latch* (`< 180 ms`, orange inner LED dot) and *Hold-to-Momentary* (`>= 180 ms`).
   - **Layer 2 (`MacThumbDialLayer::FreeJoystickBowl`, toggled by clicking the Center Button):** Dissolves all radial dividers into a **smooth, unobstructed 360° recessed Liquid Glass bowl** with concentric dotted `Fine` and `Fast` speed rings and a freely gliding $360^\circ$ analog glass thumb-stick puck connected to the center by an elastic blue vector ray. The point-nudge speed scales proportionally/quadratically toward the outer rim ($v \propto (r / R_{\max})^2$) and springs back to the center on release.
   - **Curved `↶ Undo` & `↷ Redo` Crescent Buttons (`MacThumbArcPlacement`):** Configurable between `TopCrown` (both curved along the top rim — zero thumb occlusion on smartphones), `BottomSkirt` (both curved along the bottom rim), and `TopAndBottomCaps` (`↶ Undo` at top `12:00`, `↷ Redo` at bottom `6:00`).

2. **Component B — `MacEdgeThumbwheel` (Top-Down Cylindrical Edge Scroll Wheel with Free-Spin Flywheel Inertia):**
   - Docked along the screen edge (or inline): a narrow recessed Liquid Glass capsule housing a **cylindrical rotary wheel viewed from the top/edge** (like a camera command dial or mouse scroll wheel seen from above).
   - Horizontal tactile ridges follow true cylindrical foreshortening $y(\phi) = R \cdot \sin(\phi)$ for $\phi \in [-75^\circ, +75^\circ]$: spaced wider with a bright specular highlight at the center crown ($\phi = 0^\circ$) and compressing closer together with 3D cylindrical shading toward the top and bottom edges.
   - **Free-Spin Flywheel Physics & Touch-to-Brake:**
     - Slow rolling ($|\omega| < \omega_{\text{flick}}$) steps discretely in ratchet detents and stops immediately upon thumb release.
     - Giving the wheel a flick ($|\omega_{\text{release}}| \ge \omega_{\text{flick}}$) engages **Free-Spin Flywheel inertia** with non-linear velocity boost and exponential decay $\omega(t) = \omega_0 e^{-\lambda t}$, spinning the visible ridges and incrementing values until $|\omega(t)| \le \omega_{\text{stop}}$ (or until the user touches the spinning wheel to brake it instantly via **Touch-to-Brake**), then returning to **0% idle CPU**.

---

## 2. Normative Requirements (`REQ-014-01` .. `REQ-014-05`)

### 2.1 `REQ-014-01`: Four Cardinal Semantic Directions & Diagonal Single-Thumb Combination Wedges (Layer 1)

| Sector (`MacThumbSector`) | Compass Angle | Universal Geometric Intent | Desktop Hint | `Pen` (`P`) Label | `Select` (`V`) Label | `Primitives` (`F`) / `BBox` Label | `Knife` (`E`) Label |
| :--- | :---: | :--- | :---: | :--- | :--- | :--- | :--- |
| **`North`** | `-90°` | **CONSTRAIN** | `⇧` | `∠ 45°` | `∠ Axis` | `1:1` | `∠ 15°` |
| **`NorthEast`** | `-45°` | **CONSTRAIN + ALTER** | `⇧+⌥` | `∠45°+∧Cusp` | `∠Axis+⧉Clone` | `1:1+⊙Center` | `∠15°+⊕Nodes` |
| **`East`** | `0°` | **ALTER / BREAK** | `⌥` | `∧ Cusp` | `⧉ Clone` | `⊙ Center` | `⊕ Nodes` |
| **`SouthEast`** | `+45°` | **GLIDE + ALTER** | `⌘+⌥` | `★G2+∧Cusp` | `↝Slide+1-Side` | `⇄ Shear` | `⌖Snap+⊕Nodes` |
| **`South`** | `+90°` | **GLIDE / SERVO** | `⌘` | `★ Genius` | `↝ Slide` | `⇄ Shear` | `⌖ Snap` |
| **`West`** | `180°` | **MOVE / PAN** | `␣` | `✥ Move Pt` | `✋ Pan` | `✥ Move` | `✋ Pan` |

### 2.2 `REQ-014-02`: Hybrid *Tap-to-Latch* (`< 180 ms`), *Hold-to-Momentary* (`>= 180 ms`) & Center Layer Switch

1. **Quick Tap (`< 180 ms`) on a Sector in Layer 1:** Toggles sticky latching in `latched_mask: MacThumbIntentMask` and illuminates the warm orange inner **Latch LED dot**. Multiple sectors can be latched sequentially.
2. **Hold (`>= 180 ms`) or Center Slide-Out in Layer 1:** Activates `momentary_mask` while held and clears it on pointer release.
3. **Quick Tap on the Center Hub (`r <= r_hub`):**
   - If `latched_mask` is non-empty, clears all active latches (`latched_mask = MacThumbIntentMask::NONE`).
   - If `latched_mask` is empty (or when `.layer(&mut layer)` is bound), toggles the active dial layer between **`MacThumbDialLayer::CompassModifiers` (Layer 1)** and **`MacThumbDialLayer::FreeJoystickBowl` (Layer 2)**.

### 2.3 `REQ-014-03`: Layer 2 — Unobstructed $360^\circ$ Free Analog Joystick Bowl

- When `layer == MacThumbDialLayer::FreeJoystickBowl`:
  - All radial sector dividers are hidden, rendering a smooth recessed *Liquid Glass* bowl with concentric `Fine` ($r = 0.45\,R_{\text{bowl}}$) and `Fast` ($r = 0.80\,R_{\text{bowl}}$) dotted rings.
  - Dragging anywhere inside the bowl displaces the translucent glass joystick puck to $\vec{d} = \text{clamp}(\vec{p} - \vec{c}, R_{\text{bowl}})$, draws a luminous accent vector line from $\vec{c}$ to the puck, and emits a continuous $360^\circ$ velocity-scaled `joystick_nudge_delta: Vec2`:
    $$\vec{\Delta}_{\text{nudge}} = \hat{u} \cdot \left(\frac{\|\vec{d}\|}{R_{\text{bowl}}}\right)^2 \cdot v_{\max}$$
    so small deflections near the center produce fine sub-point adjustments and outer-rim deflections produce fast translation across all $360^\circ$ angles without any directional obstacles.

### 2.4 `REQ-014-04`: `MacEdgeThumbwheel` — Top-Down Cylindrical Scroll Wheel with Free-Spin Flywheel & Touch-to-Brake

- Renders a vertical (or horizontal) recessed *Liquid Glass* slot containing a cylindrical wheel viewed from the top/edge.
- **Cylindrical Foreshortening Ridge Projection**:
  - For wheel phase $\theta_{\text{wheel}}$ and ridge pitch $\Delta\phi = 15^\circ$, every visible ridge with $\phi_k \in [-75^\circ, +75^\circ]$ is projected at vertical offset $y_k = y_c + R_{\text{cyl}} \cdot \sin(\phi_k)$ with stroke thickness and specular brightness proportional to $\cos(\phi_k)$ (widest and brightest at the center crown $\phi_k = 0^\circ$, compressed and shaded near the top/bottom edges).
- **Free-Spin Flywheel Inertia & Touch-to-Brake**:
  - Releasing a drag with $|\omega_0| \ge \omega_{\text{flick}}$ engages free-spin coasting ($\omega(t) = \omega_0 e^{-\lambda t}$) with velocity boost $(1 + k_{\text{boost}}|\omega|)$, requesting timed repaints only while $|\omega(t)| > \omega_{\text{stop}}$ and settling to zero idle repaints once stopped.
  - Pressing on the wheel while $|\omega(t)| > 0$ immediately zeroes the flywheel velocity (*Touch-to-Brake*).

### 2.5 `REQ-014-05`: Configurable `MacThumbArcPlacement` & Dark/Light *Liquid Glass* Gallery Showcase (`v0.8.0`)

- `MacContextThumbDial` supports `MacThumbArcPlacement::{ BottomSkirt, TopCrown, TopAndBottomCaps }` and renders authentic *Liquid Glass* in both `MacAppearance::DarkAqua` and `MacAppearance::Aqua`.
- `macview_gallery` (`State 1: Vector & Inspector Studio`) mounts both **`MacContextThumbDial`** (with Layer 1 $\leftrightarrow$ Layer 2 center-click switching and placement selector) and **`MacEdgeThumbwheel`** (with Free-Spin flywheel inertia) live on the vector canvas.
