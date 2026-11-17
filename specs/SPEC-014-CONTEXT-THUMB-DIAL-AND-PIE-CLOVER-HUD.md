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

## 2. Normative Requirements (`REQ-014-01` .. `REQ-014-07`)

### 2.1 `REQ-014-01`: Four Cardinal Semantic Directions & Diagonal Single-Thumb Combination Wedges (Layer 1)

| Sector (`MacThumbSector`) | Compass Angle | Universal Geometric Intent | Desktop Hint | `Pen` (`P`) Label | `Select` (`V`) Label | `Primitives` (`F`) / `BBox` Label | `Knife` (`E`) Label |
| :--- | :---: | :--- | :---: | :--- | :--- | :--- | :--- |
| **`North`** | `-90°` | **CONSTRAIN** | `⇧` | `∠ 45°` | `∠ Axis` | `1:1` | `∠ 15°` |
| **`NorthEast`** | `-45°` | **CONSTRAIN + ALTER** | `⇧+⌥` | `∠45°+∧Cusp` | `∠Axis+⧉Clone` | `1:1+⊙Center` | `∠15°+⊕Nodes` |
| **`East`** | `0°` | **ALTER / BREAK** | `⌥` | `∧ Cusp` | `⧉ Clone` | `⊙ Center` | `⊕ Nodes` |
| **`SouthEast`** | `+45°` | **GLIDE + ALTER** | `⌘+⌥` | `★G2+∧Cusp` | `↝Slide+1-Side` | `⇄ Shear` | `⌖Snap+⊕Nodes` |
| **`South`** | `+90°` | **GLIDE / SERVO** | `⌘` | `★ Genius` | `↝ Slide` | `⇄ Shear` | `⌖ Snap` |
| **`West`** | `180°` | **MOVE / PAN** | `␣` | `✥ Move Pt` | `✋ Pan` | `✥ Move` | `✋ Pan` |

### 2.2 `REQ-014-02`: Exclusive Single-Sector *Tap-to-Latch* (`< 180 ms`), *Hold-to-Momentary* (`>= 180 ms`) & Center Layer Switch

1. **Exclusive Single-Sector Latch (`latched_sector: Option<MacThumbSector>`):**
   - Because diagonal combination cushions (`NorthEast` = `⇧+⌥`, `SouthEast` = `⌘+⌥`) are dedicated physical cushions on the dial, **at most one sector cushion can be latched at any time** (exclusive radio-latch behavior).
   - **Quick Tap (`< 180 ms`) on a Sector in Layer 1:**
     - If that sector is already the active `latched_sector`, unlatches it (`latched_sector = None`, `latched_mask = MacThumbIntentMask::NONE`).
     - Otherwise, switches `latched_sector` exclusively to that tapped sector (`Some(sector)`), sets `latched_mask = sector.intent_mask()`, and illuminates the warm orange inner **Latch LED dot** (with halo) on **only that single sector**.
2. **Hold (`>= 180 ms`) or Center Slide-Out in Layer 1:** Activates `momentary_mask` for that single hovered/held sector while held and clears it on pointer release.
3. **Quick Tap on the Center Hub (`r <= r_hub`):**
   - If `latched_sector` is `Some(_)` (or `latched_mask` is non-empty), clears the active latch (`None` / `MacThumbIntentMask::NONE`).
   - If no sector is latched, toggles the active dial layer between **`MacThumbDialLayer::CompassModifiers` (Layer 1)** and **`MacThumbDialLayer::FreeJoystickBowl` (Layer 2)**.

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

### 2.6 `REQ-014-06`: Three Standalone Interchangeable Widget Configurations & Calculated Apple Vibrancy/Contrast Glass Color (`.glass_accent(Color32)`)

1. **Three Standalone, Independently Mountable & Swappable Widgets:**
   - **Configuration 1 — Standalone Rotary Wheel (`MacEdgeThumbwheel`):** Cylindrical top-down roller with $\cos(\phi)$ foreshortened ridges, illuminated end-cap accent rings, Free-Spin flywheel inertia, and Touch-to-Brake, usable either paired above the thumb dial or completely standalone.
   - **Configuration 2 — Standalone 6-Cushion Selection / Modifier Dial (`MacContextThumbDial`):** Compact circular *Liquid Glass* 6-cushion compass modifier dial with curved `↶ Undo` / `↷ Redo` crown buttons (`MacThumbArcPlacement::TopCrown`), *Tap-to-Latch*, and *Hold-to-Momentary*.
   - **Configuration 3 — Standalone $360^\circ$ Free Joystick Bowl (`MacFreeJoystickBowl`):** Dedicated standalone widget rendering the unobstructed $360^\circ$ recessed *Liquid Glass* bowl (at the exact same compact $1:1$ circular footprint as `MacContextThumbDial`), concentric `Fine` and `Fast` speed rings, curved `↶ Undo` / `↷ Redo` crown buttons, and a **5-layer 3D translucent crystal glass marble** (ambient drop shadow, refractive glass body, inner caustic glow, Fresnel rim highlight, and upper-left specular highlight dot) that glides across $360^\circ$ with quadratic radial velocity scaling.
2. **Free User-Configurable `.glass_accent(Color32)` Adapted by the Apple Calculated Color System (`MacTheme::resolve_glass_accent_on`):**
   - All three widgets (`MacEdgeThumbwheel`, `MacContextThumbDial`, and `MacFreeJoystickBowl`) accept any user-chosen `egui::Color32` via `.glass_accent(color: Color32)` (e.g., selected interactively via `MacColorWell`).
   - Instead of painting the raw `Color32` unchecked, each widget passes the user's color through `MacTheme::resolve_glass_accent_on(custom_color, surface_bg)` in `src/theme/contrast.rs`, which applies Apple CoreGraphics surface vibrancy (`plus_darker` on light `Aqua` surfaces, `plus_lighter` on dark `DarkAqua`/`Anthracite` surfaces) and `ensure_wcag_contrast` (`>= 3.0:1` UI component contrast), while resolving all cushion and bowl typography through `theme.label_on(...)`, `theme.secondary_label_on(...)`, and `MacBackgroundStyle::from_color(...)`.

### 2.7 `REQ-014-07`: Pluggable Apple SF Symbol & Custom Vector Icon Interface (`MacSFSymbol`, `MacThumbSectorIcon`), Haptic/Auditory Micro-Click Telemetry (`MacTapticEvent`) & 5-Layer 3D *Liquid Glass* Depth

1. **Pluggable Per-Sector Vector Icon Interface (`MacSFSymbol`, `MacThumbSectorIcon` & `SegmentIconFn`):**
   - Every sector descriptor [`MacThumbSectorSpec`] carries a typed `icon: MacThumbSectorIcon` alongside a concise short caption (`label: &'a str`, max 5 chars) so small radial cushions never suffer from cramped text strings or missing font glyphs:
     - `MacThumbSectorIcon::SFSymbol(MacSFSymbol)`: Official Apple SF Symbols (`AspectRatio` `"aspectratio"`, `Scope` `"scope"`, `Viewfinder` `"viewfinder"`, `PlusSquareOnSquare` `"plus.square.on.square"`, `Skew` `"skew"`, `Scribble` `"scribble"`, `Sparkles` `"sparkles"`, `PlusCircle` `"plus.circle"`, `ArrowUpAndDownAndLeftAndRight` `"arrow.up.and.down.and.left.and.right"`, `HandRaised` `"hand.raised"`, `Angle` `"angle"`, `PointTopleftDownCurvedtoPointBottomrightUp` `"point.topleft.down.curvedto.point.bottomright.up"`, `Shift` `"shift"`, `OptionKey` `"option"`, `CommandKey` `"command"`, `ControlKey` `"control"`).
     - `MacThumbSectorIcon::Cupertino(MacSymbol)`: Any embedded `CupertinoIcons` vector symbol (`SPEC-008A`).
     - `MacThumbSectorIcon::CustomFn(SegmentIconFn)`: Any host-provided `fn(&Painter, Rect, Color32)` callback (sharing the exact [`SegmentIconFn`] signature of `MacSegmentedControl`), enabling host vector editors to swap in custom tool-specific vector icons dynamically whenever the active tool changes (via `.custom_sectors(...)` or `.with_sector_icon(sector, icon)`).
2. **Haptic & Auditory Micro-Click Telemetry (`MacTapticEvent`):**
   - `MacEdgeThumbwheelResponse`, `MacContextThumbResponse`, and `MacFreeJoystickResponse` expose `pub taptic_event: Option<MacTapticEvent>`, emitting:
     - `MacTapticEvent::RotaryDetentTick` on each $15^\circ$ ridge step (including during Free-Spin flywheel coasting),
     - `MacTapticEvent::FlywheelBraked` on Touch-to-Brake contact while spinning,
     - `MacTapticEvent::SectorLatched(MacThumbSector)` and `MacTapticEvent::SectorUnlatched`,
     - `MacTapticEvent::LayerSwitched(MacThumbDialLayer)` on center-button layer toggle,
     - `MacTapticEvent::JoystickRingCrossed` when the 3D crystal marble crosses the `Fine` or `Fast` speed ring, and
     - `MacTapticEvent::HistoryStep { is_redo: bool }` on `↶ Undo` / `↷ Redo`.
3. **5-Layer 3D *Liquid Glass* Cushion, Bowl & Cylindrical Drum Depth:**
   - **Rounded Cushions:** Rendered with recessed dark outer grooves, dual-zone vertical glass sheen (bright upper specular zone + deep lower refraction zone), a crisp `1.5px` top-arc Fresnel specular highlight, and a multi-ring radial bloom + glowing LED halo on the single active sector.
   - **Recessed 3D Joystick Bowl:** Rendered with an inner top-rim shadow gradient, lower-rim upward reflection arc, dotted `Fine`/`Fast` speed rings (discrete circular dots), and a 5-layer 3D crystal glass marble with a lower caustic crescent.
   - **3D Cylindrical Drum (`MacEdgeThumbwheel`):** Rendered with multi-band vertical cylindrical shading (dark top/bottom curvature roll-off, bright center horizon reflection band, and glowing 3D end-cap rings).
