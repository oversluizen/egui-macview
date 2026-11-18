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
3. **5-Layer 3D *Liquid Glass* Cushion, Bowl & Cylindrical Drum Depth (`REQ-014-07.3` & `REQ-014-08` Mockup Parity):**
   - **Filleted 3D Annular Cushions (`BUG-DIAL-01` & `BUG-DIAL-03`):** Each of the 6 radial cushions SHALL be tessellated with rounded corner fillets (`corner_radius ≈ 6pt` at all 4 radial-arc corners) and a recessed inter-cushion groove shadow, using a vertex-colored `egui::epaint::Mesh` (dual-zone vertical/radial glass gradient from brighter top specular zone to deeper bottom refraction zone, plus a crisp `1.5px` top-arc Fresnel specular highlight and active cyan-blue inner bloom).
   - **Multi-Tier Cushion Icon + Label Typography & Glowing Amber Latch LED (`BUG-DIAL-02`):** Each cushion SHALL support the 3 presentation tiers shown in the Golden Reference Mockups (`render1_dark_mode_stand1_v3` & `render3_light_mode_aqua_stand1_v2`): upper vector icon badge, bold 2/3-letter code or primary glyph, and concise sub-caption when space/mode permits, plus a warm amber/orange inner-corner **Latch LED** (`r = 2.6pt`) surrounded by a concentric soft glow halo (`r = 6.5pt`) on the latched sector.
   - **Smooth 3D Glass Outer Bezel & Rounded Stadium `↶ Undo` / `↷ Redo` Crescents (`BUG-DIAL-04`):** The outer ring bezel SHALL render a clean, smooth 3D glass rim without noisy 24-tick compass clutter, and the `↶ Undo` and `↷ Redo` crown buttons SHALL render as separated, semicircularly rounded 3D curved glass capsules with specular top edge and Cupertino history symbols.
   - **16-Band Cosine Cylindrical Drum Shading on `MacEdgeThumbwheel` & Keyboard/AccessKit Parity (`BUG-DIAL-05`):** `MacEdgeThumbwheel` SHALL render a recessed glass housing shadow and a 16-band cosine-interpolated vertical cylindrical gradient mesh beneath the foreshortened ridges, and both `MacContextThumbDial` and `MacEdgeThumbwheel` SHALL expose semantic `egui::WidgetInfo` (`AccessKit`) and full keyboard accessibility (arrow keys / `Tab` / `Space` / `Enter` / `Escape`).

### 2.8 `REQ-014-10`: Apple Liquid Glass Optical Engine — Interactive Pointer `CASpring` Physics (`.lift` & `.highlight`), 3D Snellius Lens Refraction ($\eta = 1.48$), and Blurred Vibrancy Backdrop Sampling (`v0.9.0`)

1. **Interactive Pointer `CASpring` State (`LiquidPointerSpring` — Apple `.interactiveSpring(response: 0.15, dampingFraction: 0.86)`):**
   - `MacContextThumbDial` (and `MacFreeJoystickBowl` / `MacEdgeThumbwheel`) SHALL maintain a per-widget damped spring state in `egui` temporary memory tracking:
     - Normalized 2D pointer offset $\vec{p}(t) = (p_x(t), p_y(t)) \in [-1, 1]^2$ relative to the widget center and outer radius, integrated with `CASpring` (`stiffness = 550.0`, `damping = 38.0`),
     - Per-sector cushion hover/lift envelopes $h_k(t) \in [0, 1]$ for all 6 sectors $k \in \{0..5\}$, and undo/redo arc envelopes $h_{\text{undo}}(t), h_{\text{redo}}(t) \in [0, 1]$ (`stiffness = 420.0`, `damping = 32.0`).
   - While any spring velocity or displacement delta exceeds `1e-3`, the widget SHALL schedule `ctx.request_repaint()`; once all springs reach rest (`is_at_rest()`), zero additional repaints SHALL be scheduled (`SPEC-004` zero-idle-wakeups invariant).
2. **`.lift` — Magnetic Cushion Pull, Radial Scale Expansion & 2.5D Parallax Tilt:**
   - When sector $k$ has spring envelope $h_k(t) > 0$, its annular cushion mesh SHALL:
     - Expand radially outward by $\Delta r_k = h_k(t) \cdot \text{ring\_gap} \cdot 0.5$ and shift magnetically toward the live pointer vector $\vec{p}(t)$ by $\Delta \vec{c}_k = \vec{p}(t) \cdot h_k(t) \cdot \text{ring\_gap}$,
     - Displace the sector icon and latch LED with a 2.5D parallax factor ($1.25 \times \Delta \vec{c}_k$) and expand the ambient drop shadow underneath the lifted cushion.
3. **`.highlight` — Pointer-Reactive 3D Blinn-Phong Specular & Schlick Fresnel Rim Shading:**
   - Each vertex $v$ on the 3D annular cushion mesh carries an analytical 3D dome surface normal $\vec{N}(v) = (n_x, n_y, n_z)$ (pointing outward along the curved bevel at the inner/outer/angular edges and upward $n_z \to 1$ in the cushion interior).
   - A dynamic virtual light source $\vec{L}(\vec{p}(t)) = \text{normalize}(-0.32 + 0.65 p_x(t),\; -0.38 + 0.65 p_y(t),\; 0.78)$ illuminates $\vec{N}(v)$, modulating each vertex color via:
     - **Blinn-Phong Specular Highlight**: $I_{\text{spec}}(v) = \max(0, \vec{N}(v) \cdot \vec{H})^8$, causing the bright liquid highlight to glide across the curved glass cushion as the mouse moves,
     - **Schlick Fresnel Rim Boost**: $F(v) = R_0 + (1 - R_0)(1 - n_z)^3$ amplified on the rim edge closest to the cursor $\vec{p}(t)$.
4. **Snellius 3D Lens Refraction ($\eta_{\text{glass}} = 1.48$) & Blurred Vibrancy Backdrop Sampling:**
   - Every Liquid Glass cushion and hub lens SHALL expose a deterministic Snellius refraction mapping `refract_backdrop_point(screen_pt: Pos2, normal_xy: Vec2, thickness_pt: f32) -> Pos2` using $\Delta \vec{x}_{\text{refract}} = -\vec{N}_{xy} \cdot \left(1 - \frac{1}{\eta_{\text{glass}}}\right) \cdot \text{thickness\_pt}$ (and an optional `backdrop_texture: Option<(egui::TextureId, Rect)>` on `MacContextThumbDial` where vertex UVs are refracted by $\Delta \vec{u} = \Delta \vec{x}_{\text{refract}} / \text{backdrop\_size}$).
   - When mounted over a vector canvas (or provided a blurred vibrancy backdrop texture), lines and background pixels seen through the curved 3D cushion bevels and center hub lens refract according to Snell's law and shift dynamically as the cushion tilts under the pointer.
