# SPEC-016: Apple Liquid Glass Optical Engine, Interactive Lensing & Fluid Morphing (`v0.9.0`)

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-016 |
| **Document Title** | Apple Liquid Glass Optical Engine, Interactive Lensing & Fluid Morphing (`v0.9.0`) |
| **Document Version** | `0.9.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.9.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, Apple macOS 26 Tahoe / visionOS Liquid Glass HIG, W3C DTCG, W3C WCAG 2.1 AA |

---

## 1. Executive Summary & Architectural Scope

Apple's **Liquid Glass** design language (introduced in macOS 26 Tahoe, iOS 26, and visionOS) replaces static flat translucent surfaces with a real-time **optically active, pointer-reactive 3D metamaterial** (`NSGlassEffectView` / SwiftUI `.glassEffect()`, `GlassEffectContainer`, `.glassEffectID()`, `.glassEffectUnion()`, and `.hoverEffect(.lift / .highlight)`).

Unlike macOS's per-pixel `WindowServer` / `CABackdropLayer` post-processor—which must brute-force millions of opaque pixels per frame across unknown third-party windows—`egui-macview` executes Liquid Glass at the **analytical vector, signed-distance-field (SDF), and per-vertex `egui::epaint::Mesh` level in pure Rust**, achieving visual parity with Apple's optical pipeline at `< 0.05 ms` CPU frame time and `0.0%` idle CPU/GPU utilization (`SPEC-004` zero-idle-wakeups invariant).

This specification anchors the complete **7-part Apple Liquid Glass Optical & Interactive Physics Architecture (`REQ-016-01` through `REQ-016-07`)** across both radial studio HUDs (`MacContextThumbDial`, `MacFreeJoystickBowl`, `MacEdgeThumbwheel`) and standard AppKit/SwiftUI controls (`MacButtonCell`, `MacSegmentedControl`, `MacSlider`, `MacToggle`, `MacToolbar`, `MacPopoverSurface`, and `MacVisualEffectView`).

---

## 2. Normative Requirements (`REQ-016-01` through `REQ-016-07`)

### 2.1 `REQ-016-01`: Analytical 3D Dome & Capsule Surface-Normal Field $\vec{N}(u, v) = (n_x, n_y, n_z)$

1. **Signed Distance Field (SDF) Bevel Profile:**
   - Every Liquid Glass element (whether a rounded capsule rect $R$ with corner radius $r_c$ or an annular filleted sector wedge $\Omega_k$ of [`MacContextThumbDial`]) defines a boundary signed distance $d(\vec{x}) \le 0$ on its interior ($d = 0$ on the perimeter, $d < 0$ inside).
   - Over the peripheral bevel width $w_{\text{bevel}} > 0$ (resolved from W3C DTCG tokens), the normalized interior penetration is:
     $$s(\vec{x}) = \text{clamp}\!\left(\frac{-d(\vec{x})}{w_{\text{bevel}}},\; 0.0,\; 1.0\right), \qquad \theta(\vec{x}) = \text{smoothstep}(0.0, 1.0, s(\vec{x})) \cdot \frac{\pi}{2}$$
   - The analytical 3D outward surface normal $\vec{N}(\vec{x}) = (n_x, n_y, n_z)$ on the curved glass dome/bevel is:
     $$\vec{N}_{xy}(\vec{x}) = \nabla d(\vec{x}) \cdot \cos(\theta(\vec{x})), \qquad n_z(\vec{x}) = \sin(\theta(\vec{x}))$$
     such that $\|\vec{N}(\vec{x})\|_2 = 1$, pointing horizontally outward ($\vec{N}_{xy} = \nabla d$, $n_z = 0$) at the extreme glass rim and vertically toward the viewer ($\vec{N} = (0, 0, 1)$) across the central plateau.

---

### 2.2 `REQ-016-02`: Snellius 3D Lens Refraction ($\eta_{\text{glass}} = 1.48$), Chromatic Edge Dispersion & Blurred Vibrancy Backdrop Sampling

1. **Snell's Law Vector & UV Refraction Mapping:**
   - For an incident view ray $\vec{I} = (0, 0, -1)$ entering a Liquid Glass element of optical thickness $h_{\text{glass}}$ and refractive index $\eta_{\text{glass}} = 1.48$ (crown/borosilicate optical glass token), the lateral refraction displacement vector in screen points is:
     $$\Delta \vec{x}_{\text{refract}}(\vec{x}) = -\vec{N}_{xy}(\vec{x}) \cdot \left(1.0 - \frac{1.0}{\eta_{\text{glass}}}\right) \cdot h_{\text{glass}} \cdot \left(1.0 + 0.35\,(1.0 - n_z(\vec{x}))^2\right)$$
   - `egui-macview` SHALL expose:
     - `refract_point_through_glass(pt: Pos2, center: Pos2, radius: f32, normal_xy: Vec2, thickness_pt: f32) -> Pos2` for zero-cost analytical refraction of underlying vector canvas geometry (grid lines, Bézier contours, selection handles), AND
     - Per-vertex UV refraction $\vec{uv}_{\text{refracted}} = \vec{uv}_{\text{screen}} + \Delta \vec{x}_{\text{refract}} / \vec{S}_{\text{backdrop}}$ on any `egui::epaint::Mesh` bound to a blurred backdrop texture (`backdrop_texture: Option<(egui::TextureId, Rect)>`).
2. **Quarter-Resolution Dual-Kawase Gaussian Backdrop Blur & $1.5\times$ Vibrancy Boost:**
   - When a host canvas or `MacVisualEffectView` (`MacVisualEffectMode::LiquidGlass`) captures or synthesizes a backdrop image, `egui-macview` SHALL provide a deterministic pure-Rust downsampled ($1/4 \times 1/4$) Gaussian/Kawase blur + $1.5\times$ saturation vibrancy booster (`build_liquid_glass_backdrop_image`) so background content seen through the glass exhibits authentic blurred translucency and lens magnification at the curved bevels.
3. **Subtle Chromatic Prismatic Rim Dispersion ($\eta_R = 1.47, \eta_G = 1.48, \eta_B = 1.50$):**
   - Along the high-curvature rim ($n_z(\vec{x}) < 0.65$), the inner Fresnel caustic tint subtly shifts toward warm cyan/azure on the light-facing rim and warm amber/violet on the opposite rim, matching Apple's chromatic dispersion on curved Liquid Glass edges.

---

### 2.3 `REQ-016-03`: Interactive Pointer `CASpring` Physics (`.hoverEffect(.lift)` & `.hoverEffect(.highlight)`)

1. **Per-Widget Damped Spring Field (`LiquidPointerSpring`):**
   - Every Liquid Glass interactive control maintains a damped spring state in `egui` memory driven by [`CASpring`] (`src/motion/mod.rs`) using Apple's published interactive spring parameters:
     - **Pointer Tracking Spring (`.interactiveSpring(response: 0.15, dampingFraction: 0.86)`)**: tracks normalized 2D cursor position $\vec{p}(t) = (p_x(t), p_y(t)) \in [-1, 1]^2$ relative to the widget center (`stiffness = 550.0`, `damping = 38.0`, `mass = 1.0`).
     - **Cushion / Capsule Lift Envelope Spring (`.spring(response: 0.28, dampingFraction: 0.78)`)**: tracks hover/press lift $h_k(t) \in [0, 1]$ per cushion or control (`stiffness = 420.0`, `damping = 32.0`, `mass = 1.0`).
   - **Zero Idle Wakeups (`SPEC-004` Invariant):** `ctx.request_repaint()` is called **if and only if** the pointer moved inside the interactive field during the current frame OR any spring in `LiquidPointerSpring` has $|\dot{x}(t)| > 10^{-3}$ or $|x(t) - x_{\text{target}}| > 10^{-3}$. Once settled, `0` repaints are requested.
2. **`.lift` — Magnetic Pull, Radial/Capsule Expansion & 2.5D Parallax Tilt:**
   - Under hover/press lift $h_k(t) > 0$, the element's mesh vertices:
     - Expand radially/outward by $\Delta r_k = h_k(t) \cdot \Delta r_{\max}$ (`1.5–2.0 pt`),
     - Shift magnetically toward the live pointer vector $\vec{p}(t)$ by $\Delta \vec{c}_k = \vec{p}(t) \cdot h_k(t) \cdot d_{\text{mag}}$ (`1.5–2.5 pt`),
     - Displace foreground vector glyphs, SF Symbols, and Latch LEDs with a $1.25\times$ 2.5D parallax multiplier so the icon floats above the glass base while the ambient drop shadow expands underneath.
3. **`.highlight` — Pointer-Reactive 3D Blinn-Phong Specular Glint & Schlick Fresnel Rim:**
   - The virtual directional light vector $\vec{L}(\vec{p}(t))$ is dynamically steered by the spring-filtered pointer position $\vec{p}(t)$:
     $$\vec{L}(\vec{p}(t)) = \text{normalize}\!\left(-0.32 + 0.65\,p_x(t),\; -0.38 + 0.65\,p_y(t),\; 0.78\right)$$
   - Halfway vector $\vec{H} = \text{normalize}(\vec{L}(\vec{p}(t)) + (0, 0, 1))$ produces:
     - **Blinn-Phong Specular Glint**: $I_{\text{spec}}(v) = \max(0.0, \vec{N}(v) \cdot \vec{H})^8$, causing the liquid highlight to glide across the 3D glass cushion in real time as the mouse moves,
     - **Directional Schlick Fresnel Rim**: $F(v) = \left(R_0 + (1.0 - R_0)(1.0 - n_z(v))^3\right) \cdot \left(0.65 + 0.35\,\max(0.0, \vec{N}_{xy}(v) \cdot \vec{p}(t))\right)$, brightening the glass rim closest to the cursor.

---

### 2.4 `REQ-016-04`: Fluid Selection Indicator Morphing (`.glassEffectID` in `MacSegmentedControl`, `MacTabView` & `MacContextThumbDial`)

1. **Continuous `CASpring` Capsule/Cushion Morphing:**
   - When the active or hovered selection changes from index $i$ to index $j$ in `MacSegmentedControl`, `MacTabView`, or across the 6 radial cushions of `MacContextThumbDial`:
     - Rather than teleporting instantaneously in 1 frame, the active Liquid Glass highlight capsule/bloom interpolates its center/angle and extent using a damped `CASpring` (`response = 0.26s`, `dampingFraction = 0.80`), producing a continuous liquid transfer between segments.

---

### 2.5 `REQ-016-05`: `GlassEffectContainer` & Smooth-Min SDF Metaball Merging (`.glassEffectUnion`, $k$-Smooth Union)

1. **Polynomial Smooth-Min ($s_{\min}$) Metaball SDF Union:**
   - When two or more adjacent Liquid Glass capsules/buttons $A$ and $B$ share a `glass_union_group_id` inside a `GlassEffectContainer` and their edge separation falls below the coalescing radius $r_{\text{merge}}$ (`8.0–12.0 pt`):
     - Their signed distance fields $d_A(\vec{x})$ and $d_B(\vec{x})$ combine via the $C^1$-continuous polynomial smooth-minimum operator:
       $$h = \text{clamp}\!\left(0.5 + 0.5\,\frac{d_B - d_A}{r_{\text{merge}}},\; 0.0,\; 1.0\right), \qquad d_{\cup}(\vec{x}) = \text{lerp}(d_B, d_A, h) - r_{\text{merge}}\,h(1.0 - h)$$
     - During animated expansion or proximity transitions, the two glass pills form a smooth liquid bridge before separating or coalescing into a single capsule with a unified Fresnel specular rim.

---

### 2.6 `REQ-016-06`: Scroll-Edge & Canvas Backdrop Luminance Adaptation (`SPEC-002A` + `SPEC-016` Dynamic Contrast)

1. **Local Under-Glass Luminance Sampling (`sample_backdrop_luminance_at`):**
   - Liquid Glass floating controls (`MacToolbar`, `MacContextThumbDial`, floating HUD pills) query the local backdrop relative luminance $Y_{\text{local}} \in [0, 1]$ under each control's screen `Rect` (from the registered surface registry `active_surface_at(rect.center())` or the bound `backdrop_texture`).
   - Foreground labels and vector SF Symbols automatically transition between high-contrast dark ink and crisp white ink with subtle caustic text shadow via `MacTheme::label_on(sampled_surface_color)`, guaranteeing WCAG 2.1 AA contrast ($\ge 4.5:1$ for text, $\ge 3.0:1$ for vector icons) even when a bright canvas artboard scrolls or pans directly beneath a dark-mode Liquid Glass HUD.

---

### 2.7 `REQ-016-07`: `MacOsVersionProfile::MacOs26Tahoe` Concentric Capsule Geometry vs. `MacOs14Sonoma` NIB Geometry Preservation

1. **Strict Separation of Geometry Regimes:**
   - **`LayoutPolicy::ExactAppleNib` & `MacOsVersionProfile::MacOs14Sonoma` / `MacOs15Sequoia`:**
     - Preserves 100% 1:1 Xcode `.nib` coordinate bijection (`<= 0.5 pt`, `REQ-013-08.1`) and `2.0 pt` `-[NSTextFieldCell drawingRectForBounds:]` compact inspector geometry (`REQ-013-08.2`). Zero unexpected inflation or capsule rounding is ever applied to exact `.nib` inspector fields.
   - **`MacOsVersionProfile::MacOs26Tahoe` & Liquid Glass HUD Controls:**
     - Applies concentric capsule radii ($R_{\text{outer}} = H / 2$, $R_{\text{inner}} = R_{\text{outer}} - \text{padding}$), 3D Snellius lens bevels (`REQ-016-01..02`), pointer-reactive `.lift` & `.highlight` springs (`REQ-016-03`), fluid selection morphing (`REQ-016-04`), and `.glassEffectUnion` metaball coalescing (`REQ-016-05`).

---

## 3. Normative Verification Oracles (`tests/spec016_liquid_glass_optical_engine_tests.rs`)

| Test Case ID | Requirement | Verification Oracle |
| :--- | :--- | :--- |
| **`tc_016_01_analytical_3d_dome_normals_and_snellius_refraction`** | `REQ-016-01`, `REQ-016-02` | Verifies that $\|\vec{N}(\vec{x})\|_2 = 1.0 \pm 10^{-4}$ across all bevel and interior points, $n_z = 0$ at the rim and $n_z = 1$ at the center plateau, and `refract_point_through_glass` bends background coordinates according to Snell's law ($\eta = 1.48$) at the curved bevel while leaving the flat center unshifted. |
| **`tc_016_02_pointer_reactive_caspring_lift_highlight_and_zero_idle_wakeups`** | `REQ-016-03` | Verifies on a CPU framebuffer that moving the pointer toward `MacThumbSector::North` vs. `MacThumbSector::South` shifts the 3D specular highlight intensity and lifts the hovered cushion mesh toward the pointer via `CASpring`, and that once the spring settles (`t > 0.45s`), `ctx.has_requested_repaint()` is `false` (zero idle wakeups). |
| **`tc_016_03_blurred_vibrancy_backdrop_and_smooth_min_metaball_union`** | `REQ-016-02`, `REQ-016-05`, `REQ-016-06` | Verifies that `build_liquid_glass_backdrop_image` produces a blurred, vibrancy-boosted backdrop texture refracted through the cushion mesh UVs, that `sdf_smooth_union` forms a continuous bridge ($d_{\cup}(\vec{x}_{\text{mid}}) < 0$) when two glass capsules approach within $r_{\text{merge}}$, and that foreground icon contrast remains $\ge 3.0:1$ over both dark and bright canvas regions beneath the dial. |
