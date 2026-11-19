# SPEC-016: Apple Liquid Glass Optical Engine, 15-Iteration MVC Roadmap & 52-Widget Environmental Interaction Matrix (`v0.9.0`)

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-016 |
| **Document Title** | Apple Liquid Glass Optical Engine, 15-Iteration MVC Roadmap & 52-Widget Environmental Interaction Matrix (`v0.9.0`) |
| **Document Version** | `0.9.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.9.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011 (MVC Separation), ISO/IEC/IEEE 29119-3:2021, Apple macOS 26 Tahoe / visionOS Liquid Glass HIG, W3C DTCG, W3C WCAG 2.1 AA |

---

## 1. Executive Summary & Architectural Philosophy

Apple's **Liquid Glass** design language (macOS 26 Tahoe, iOS 26, visionOS) treats user interface controls not as isolated 2D painted rectangles, but as a **coupled optical and physical ecosystem**. Every control in a Liquid Glass scene participates in three formal relationships:
1. **Base Material State (*Self / Resting Geometry & Optics*)**: Its 2D/3D Signed Distance Field (SDF) silhouette, surface normal field $\vec{N}(\vec{x}) = (n_x, n_y, n_z)$, internal sub-surface caustic tint, and resting Fresnel rim.
2. **Outward Emission (*Effect on Neighboring Controls & Parent Surface*)**: How the control emits light, shadow, refractive surface registration (`register_surface_rect`), fluid selection transfer (`DualSpringMorphBlob`), or metaball coalescing bridges (`.glassEffectUnion`) into its surrounding environment.
3. **Inward Reception (*Response to Pointer Field, Backdrop & Layout Regime*)**: How the control deforms, tilts, and shifts its specular highlight under the global 2D pointer light field $\vec{p}(t)$, how it adapts foreground contrast to underlying canvas luminance (`sample_backdrop_luminance_at`), and how it preserves 1:1 coordinate bijection under `LayoutPolicy::ExactAppleNib` (`MacOs14Sonoma`).

---

## 2. The 15-Iteration MVC Execution Architecture (`M1..M3` + `W1-V/C..W6-V/C`)

To prevent ad-hoc procedural hacks and guarantee mathematical, visual, and behavioral verification at every layer, the Liquid Glass transformation is decomposed into **15 closed-loop COTS iterations** (`Spec Anchor -> RED COTS Test -> GREEN Implementation -> Retina PNG Verification`):

| Iteration | Layer / MVC Role | Scope & Widget Family | Primary Verification Oracle |
| :--- | :--- | :--- | :--- |
| **Iter 1 (`M1`)** | **Model — Geometry & SDF** | Analytical 2D/3D SDF primitives: `sdf_annular_rounded_sector` (constant-radius pebble cushions + optical centroid), `sdf_curved_stadium_arc` (semicircular Undo/Redo capsules + arc tangent frame), `sdf_rounded_capsule`, `sdf_smooth_union` ($C^1$ polynomial metaball), and 3D dome normal $\vec{N}(\vec{x})$ | `tc_016_m1_sdf_annular_sector_stadium_arc_and_3d_normals` |
| **Iter 2 (`M2`)** | **Model — Optics & Material** | Snell's Law refraction ($\eta = 1.48$) with chromatic dispersion ($\eta_R=1.47, \eta_G=1.48, \eta_B=1.50$), Schlick Fresnel + Blinn-Phong specular + sub-surface caustic bloom, `ProximityLightField` (cross-widget rim illumination), and quarter-res Dual-Kawase Gaussian blur + $1.5\times$ vibrancy backdrop | `tc_016_m2_snellius_chromatic_fresnel_proximity_and_kawase_backdrop` |
| **Iter 3 (`M3`)** | **Model — Coupled `CASpring` Dynamics** | `DualSpringMorphBlob` (leading-edge + trailing-edge velocity-stretched liquid droplet transfer), `AnisotropicCushionSpring` (radial/magnetic cushion lift + 2.5D parallax + jelly compression), W3C DTCG token synchronization, and `is_at_rest() -> Duration::MAX` zero-idle-wakeups proof | `tc_016_m3_dual_spring_morph_blob_anisotropic_lift_and_zero_idle` |
| **Iter 4 (`W1-V`)** | **View — Family 1 (Radial Pie HUD)** | `MacContextThumbDial` View: 8-cushion $360^\circ$ rosette (`N, NE, E, SE, S, SW, W, NW`) in a recessed 3D dish, $10 \times 20$ 2D-grid cushion meshes, mandatory 3-tier typography (`Vector SF Symbol` + `Bold Label` + `Shortcut`) at the SDF optical centroid, 3-layer amber Latch LED, and semicircular stadium `"↶ Undo"` / `"↷ Redo"` capsules with arc-rotated typography (`-30°` / `+30°`) | `tc_016_w1v_thumb_dial_8_cushion_mesh_3_tier_labels_and_rotated_stadium_arcs` |
| **Iter 5 (`W1-C`)** | **Controller — Family 1 (Radial Pie HUD)** | `MacContextThumbDial` Controller: Exact SDF hit-testing on rounded cushions & curved stadium capsules, polar `DualSpringMorphBlob` gliding between hovered sectors, *Tap-to-Latch* (`< 180 ms`) vs. *Hold-to-Momentary*, `MacTapticEvent` synthesis, and full keyboard/`AccessKit` navigation | `tc_016_w1c_thumb_dial_sdf_hit_polar_morph_latch_and_a11y` |
| **Iter 6 (`W2-V`)** | **View — Family 2 (Bimanual Drum & Bowl)** | `MacEdgeThumbwheel` (Component B: recessed glass trough, outer cyan/white bloom, illuminated end-caps, 16-band cosine drum + $\cos(\phi)$ ridges) & `MacFreeJoystickBowl` (Stand 2: recessed $360^\circ$ dish, `Fine`/`Fast` dotted rings, vector directional tether, 5-layer refractive crystal marble) | `tc_016_w2v_edge_thumbwheel_drum_and_free_joystick_crystal_marble` |
| **Iter 7 (`W2-C`)** | **Controller — Family 2 (Bimanual Drum & Bowl)** | `MacEdgeThumbwheel` Free-Spin flywheel inertia + Touch-to-Brake coupled to specular ridge shift, and `MacFreeJoystickBowl` quadratic radial velocity + ring-crossing `MacTapticEvent` + caustic shadow displacement | `tc_016_w2c_flywheel_brake_and_joystick_bowl_physics` |
| **Iter 8 (`W3-V`)** | **View — Family 3 (Morphing Selection Bars)** | `MacSegmentedControl`, `MacTabView`, and `MacTableActionBar` View: 5-layer 3D Liquid Glass active/hover selection pill rendered via `DualSpringMorphBlob` (stretching between segments) and `.glassEffectUnion` metaball bridging | `tc_016_w3v_segmented_and_tab_morphing_liquid_capsule_view` |
| **Iter 9 (`W3-C`)** | **Controller — Family 3 (Morphing Selection Bars)** | `MacSegmentedControl`, `MacTabView`, and `MacTableActionBar` Controller: Hover pre-lensing attraction toward neighboring segments, drag-slide scrubbing across segments, and `PressAndHold`/`PullDown` menu triggers | `tc_016_w3c_segmented_and_tab_drag_scrub_and_pre_lensing` |
| **Iter 10 (`W4-V`)** | **View — Family 4 (Action & Value Primitives)** | `MacButtonCell`, `MacToggle`, `MacSwitch`, `MacCheckbox`, `MacRadioButton`, `MacSlider`, `MacCircularSlider`, `MacStepper`, `MacPopUpButton`, `MacComboBox`, `MacColorWell`, `MacHelpButton`, `MacInlineActionButton`, `MacDisclosureButton`, `MacRatingIndicator` View: 3D Liquid Glass capsule/circle meshes, refractive glass droplet slider/toggle thumbs, and `ProximityLightField` rim illumination | `tc_016_w4v_action_and_value_primitives_liquid_glass_meshes` |
| **Iter 11 (`W4-C`)** | **Controller — Family 4 (Action & Value Primitives)** | Family 4 Controller: `AnisotropicCushionSpring` `.lift` and jelly-press on click/drag, magnetic thumb tracking on `MacSlider`/`MacToggle`, and strict `LayoutPolicy::ExactAppleNib` Sonoma geometry preservation (`REQ-016-07`) | `tc_016_w4c_action_and_value_primitives_spring_controller` |
| **Iter 12 (`W5-V`)** | **View — Family 5 (Text, Inspector & Form Controls)** | `MacInputCell`, `MacSearchField`, `MacTokenField`, `MacTextView`, `MacLabel`, `MacOriginMatrix`, `MacTagColorPicker`, `MacRuleEditor`, `MacDatePicker`, `MacPathControl`, `MacForm` View: Glowing 3D Liquid Glass active/focused inspector row capsule (matching `X: 350.0` in `SelectionInfo` mockup), recessed glass input wells, and token pills | `tc_016_w5v_inspector_selection_pill_and_input_wells` |
| **Iter 13 (`W5-C`)** | **Controller — Family 5 (Text, Inspector & Form Controls)** | Family 5 Controller: Fluid `DualSpringMorphBlob` transfer of the glowing focus/hover capsule from row to row on `Tab`/`Shift+Tab` or pointer traversal, while preserving `2.0 pt` `-[NSTextFieldCell drawingRectForBounds:]` zero-truncation invariant | `tc_016_w5c_inspector_focus_morph_and_zero_truncation` |
| **Iter 14 (`W6-V`)** | **View — Family 6 (Floating Surfaces, Navigation & Media)** | `MacPopoverSurface`, `MacPopover`, `MacPanel`, `MacCard`, `MacBox`, `MacFloatingInfoBox`, `MacToolbar`, `MacMenuBar`, `NSMenu`, `MacCommandPalette`, `MacCanvasFooter`, `MacContextMenu`, `MacToast`, `MacPlayerView`, `MacTrimScrubber`, `MacLevelIndicator`, `MacProgressIndicator`, `MacTableView`, `MacOutlineView`, `MacCollectionView`, `MacThumbnailCell`, `MacImageView`, `MacScrollView`, `MacSplitView`, `MacStackView`, `MacFixedView`, `MacRow` View: Dual-Kawase blurred vibrancy backdrop sheets, Snellius edge refraction, and scroll-edge luminance contrast adaptation | `tc_016_w6v_floating_surfaces_navigation_and_media_refraction` |
| **Iter 15 (`W6-C`)** | **Controller — Family 6 (Floating Surfaces, Navigation & Media)** | Family 6 Controller: Cross-surface `register_surface_rect` propagation, `ProximityLightField` propagation from parent toolbar/popover to child controls, `CASpring` sheet/toast transitions, full 31-suite regression, Retina PNG Golden Master comparison, and GitHub Pages WASM deployment | `tc_016_w6c_full_ecosystem_integration_and_golden_master` |

---

## 3. Complete 52-Widget Environmental Interaction Matrix (*Base Appearance, Outward Emission & Inward Reception*)

Every public widget and layout container in `egui-macview` (52 total across the 6 families) is governed by the following three-column behavioral contract:

### 3.1 Family 1 & Family 2: Bimanual Radial Studio HUDs (3 Widgets)

| # | Widget | 1. Base Appearance (*Self / Resting State*) | 2. Outward Emission (*Behavior Toward Environment*) | 3. Inward Reception (*Response From Environment*) |
| :--- | :--- | :--- | :--- | :--- |
| **1** | **`MacContextThumbDial`** | Recessed 3D circular glass dish containing an 8-cushion $360^\circ$ pebble rosette (`N, NE, E, SE, S, SW, W, NW`), center reticle hub lens, 3-tier cushion typography (`Icon` + `Bold Label` + `Shortcut`), 3-layer amber Latch LED, and semicircular `"↶ Undo"` / `"↷ Redo"` crown stadium capsules. | Refracts underlying vector canvas lines/backdrop through its 8 curved cushion bevels and hub lens ($\eta = 1.48$); emits `MacTapticEvent` micro-clicks, `effective_mask` modifier chords, and `ProximityLightField` glow onto its neighbor `MacEdgeThumbwheel`. | Tracks 2D pointer via `AnisotropicCushionSpring` (`.lift` radial expansion + $1.25\times$ 2.5D icon parallax); glides polar `DualSpringMorphBlob` highlight from cushion to cushion; adapts glass accent via `resolve_glass_accent_on` against canvas luminance. |
| **2** | **`MacEdgeThumbwheel`** | Wide horizontal cylindrical roller inside a deep rounded glass trough with soft outer cyan/white bloom, illuminated left/right end-cap rings, 16-band cosine cylindrical shading, and $\cos(\phi)$-foreshortened tactile ridges. | Emits angular delta, Free-Spin flywheel velocity $\omega(t)$, `MacTapticEvent::RotaryDetentTick` / `FlywheelBraked`, and casts a soft specular reflection onto the top crown of `MacContextThumbDial` when paired. | Shifts its cylindrical specular highlight band toward the pointer vector $\vec{p}(t)$; brakes immediately on pointer touch (`Touch-to-Brake`); adapts end-cap neon glow to `.glass_accent(Color32)`. |
| **3** | **`MacFreeJoystickBowl`** | Unobstructed $360^\circ$ recessed 3D glass dish with concentric `Fine` ($0.45 R$) and `Fast` ($0.82 R$) dotted rings, center origin dot, directional blue vector tether, and a 5-layer 3D refractive crystal marble puck. | Emits continuous quadratic 2D nudge vector $\vec{v}_{\text{nudge}}$, `MacTapticEvent::JoystickRingCrossed`, and casts a moving caustic drop shadow + refractive lens distortion onto the recessed bowl floor beneath the marble. | Marble puck follows pointer drag across $360^\circ$ and snaps back to center origin via underdamped `CASpring` on release; specular pinpoint on marble tracks the pointer light angle. |

---

### 3.2 Family 3: Fluid Morphing Selection Bars (3 Widgets)

| # | Widget | 1. Base Appearance (*Self / Resting State*) | 2. Outward Emission (*Behavior Toward Environment*) | 3. Inward Reception (*Response From Environment*) |
| :--- | :--- | :--- | :--- | :--- |
| **4** | **`MacSegmentedControl`** | Recessed translucent glass track housing $N$ segments, with the selected segment covered by a raised 5-layer 3D Liquid Glass capsule (`Fresnel rim + dome gradient + drop shadow`). | Active pill emits a `DualSpringMorphBlob` liquid bridge toward any hovered sibling segment and suppresses adjacent 1px dividers; emits `MacSegmentEvent` and optional `PressAndHold`/`PullDown` popover menus. | Active capsule stretches and flows (`leading/trailing CASpring`) when selection changes; hovered unselected segment pre-illuminates its Fresnel rim via `ProximityLightField`; preserves compact square bezel in `ExactAppleNib`. |
| **5** | **`MacTabView`** | Top segmented Liquid Glass tab bar sitting above a recessed `MacBox`-style content surface (`Bezel` or `Borderless` style). | Registers its inner tab content rect in the surface registry (`register_surface_rect`) so all child widgets inside the active tab automatically adapt their WCAG foreground contrast; morphs active tab pill across tab headers. | Tab indicator capsule flows via `DualSpringMorphBlob` on tab switch; child `<tabViewItem>` nodes switch visibility declaratively (`VisibilityRule::WhenTabSelected`). |
| **6** | **`MacTableActionBar`** | Compact bottom-attached or standalone strip of icon action cells (`+`, `-`, `⚙` / action menu) with crisp 1pt hairline borders. | Emits semantic add/remove/action events and triggers green/red semantic pill glow (`semantic_add_hover_colors` / `semantic_remove_hover_colors`) on hover. | Each cell illuminates its local Fresnel border as the pointer approaches and depresses with a crisp `CASpring` micro-scale on click. |

---

### 3.3 Family 4: Action & Value Primitives (15 Widgets)

| # | Widget | 1. Base Appearance (*Self / Resting State*) | 2. Outward Emission (*Behavior Toward Environment*) | 3. Inward Reception (*Response From Environment*) |
| :--- | :--- | :--- | :--- | :--- |
| **7** | **`MacButtonCell`** (`MacButton`) | Raised 3D Liquid Glass capsule (`MacOs26Tahoe`) or rounded AppKit bezel (`MacOs14Sonoma`) with top specular rim, dome gradient, and drop shadow. | Emits `.glassEffectUnion` smooth-min metaball bridge to adjacent buttons in a shared group; casts expanding drop shadow onto parent surface when lifted. | Lifts (`+3%` scale / magnetic pull) and glides its Blinn-Phong specular glint with pointer $\vec{p}(t)$; preserves exact `[x, y, w, h]` alignment rect in `ExactAppleNib`. |
| **8** | **`MacToggle`** | Pill-shaped track (accent-filled when ON, recessed glass when OFF) containing a raised 3D Liquid Glass circular/oval thumb lens. | Emits boolean state transition and casts a warm accent caustic glow from the track onto the inner rim of the sliding glass thumb. | Thumb stretches horizontally (`anisotropic squash/stretch`) while sliding across the track via `CASpring`; specular rim tracks pointer proximity. |
| **9** | **`MacSwitch`** | Compact AppKit switch alias sharing the [`MacToggle`] 3D Liquid Glass track-and-lens architecture across `Regular`, `Small`, and `Mini` sizes. | Emits state change and focus-ring halo onto parent row. | Responds to click, drag-throw, and `Space` key with `CASpring` thumb translation. |
| **10** | **`MacCheckbox`** | Rounded square glass well (`14×14pt` Regular, `12×12pt` Small, `10×10pt` Mini) with accent fill and crisp analytical vector checkmark when checked. | Emits boolean/indeterminate state and shares hover highlight with its attached `MacLabel` caption. | Illuminates top specular rim on hover; scales checkmark vector stroke in via `CASpring`; resolves label ink via `theme.label_on(active_surface)`. |
| **11** | **`MacRadioButton`** | Circular 3D glass well with accent fill and raised inner white specular core dot when selected. | Emits mutual-exclusion selection within a radio group. | Core dot blooms radially via `CASpring` on selection; rim brightens under `ProximityLightField`. |
| **12** | **`MacSlider`** | Recessed glass track (`4.0pt` neutral or accent-filled) with optional tick marks and a raised 3D refractive Liquid Glass capsule/circle thumb. | Emits continuous value changes and casts a subtle refractive caustic highlight onto the track directly beneath the thumb. | Thumb lifts and enlarges slightly into a translucent glass lens while dragged, with its specular highlight tracking the pointer. |
| **13** | **`MacCircularSlider`** | Circular rotary glass dial with radial value arc and a 3D specular dimple indicator at the current angle. | Emits continuous angular value in `[min..=max]`. | Specular rim and dimple track pointer angle; responds to vertical/circular drag and scroll wheel. |
| **14** | **`MacStepper`** | Vertical dual-chevron (`⌃` / `⌄`) split glass capsule paired beside numeric [`MacInputCell`] fields. | Increments/decrements paired numeric field with `Shift` ($10\times$) and `Cmd` ($0.1\times$) multipliers; caret casts `caret_micro_shadow`. | Pressed half-capsule depresses inward with top/bottom specular inversion while the unpressed half remains raised. |
| **15** | **`MacPopUpButton`** | Raised glass control bezel with left value label and right accent/neutral dual-chevron (`⌄`/`⌃`) badge (`accent_badge`). | Spawns a floating `MacPopoverSurface` / `MacContextMenu` menu layer clamped to viewport bounds, casting backdrop shadow over underlying controls. | Bezel rim tracks pointer highlight; active open state latches accent glow until menu dismissal. |
| **16** | **`MacComboBox`** | Hybrid recessed text-input well on the left fused with a raised Liquid Glass dropdown trigger cap on the right. | Opens viewport-clamped autocomplete/option popover list and commits selected string to the edit buffer. | Focus ring wraps the unified capsule; right trigger cap independently highlights under pointer hover. |
| **17** | **`MacColorWell`** | Framed glass swatch pill displaying a live color tile with diagonal alpha checkerboard and inner specular border. | Opens the non-modal floating 2D Saturation-Value + 1D Hue `egui::epaint::Mesh` color picker popover (`REQ-002-11`) and feeds live `Color32` (e.g. `.glass_accent`) to target widgets. | Unlocks achromatic colors (`unlock_achromatic_hsva`) when hue bar is dragged; swatch bezel lifts on hover. |
| **18** | **`MacHelpButton`** | Circular 3D Liquid Glass question-mark button (`?` vector stroke or `Inter-SemiBold` glyph) with outer specular rim. | Triggers contextual help popover or documentation callback. | Exhibits circular radial lens highlight and `CASpring` lift on pointer hover. |
| **19** | **`MacInlineActionButton`** | Borderless or subtle pill-framed micro-action icon (`+`, `-`, `↻`, `✕`, chevron) inside inspector rows and headers. | Emits row action; `+` and `-` variants emit semantic green/red translucent pill halos (`REQ-015-02`). | Transitions from `0%` background fill at rest to `inspector_hover_fill` / semantic tint via `CASpring` on pointer hover. |
| **20** | **`MacDisclosureButton`** | Standalone pill/circular chevron button (`▶` / `▼`) for expanding/collapsing detail sections. | Drives sibling detail container visibility and rotates chevron angle via `CASpring`. | Pill bezel illuminates on pointer approach; chevron rotates smoothly between $0^\circ$ and $90^\circ$. |
| **21** | **`MacRatingIndicator`** | Horizontal row of interactive star glyphs (`★` / `☆`) or level dots. | Emits integer/fractional rating value. | Hovered star and all preceding stars ripple-scale upward in sequence via staggered `CASpring` offsets. |

---

### 3.4 Family 5: Text, Inspector & Form Controls (11 Widgets)

| # | Widget | 1. Base Appearance (*Self / Resting State*) | 2. Outward Emission (*Behavior Toward Environment*) | 3. Inward Reception (*Response From Environment*) |
| :--- | :--- | :--- | :--- | :--- |
| **22** | **`MacInputCell`** | Recessed glass text well with `1.0pt` `input_top_inset_shadow`, `2.0pt` `NSTextFieldSquareBezel` corner radius (in `.nib`) or concentric capsule (in `SelectionInfo` HUD), transactional edit buffer, and optional unit suffix + stepper. | When focused inside an inspector/form row, emits `focused_row_fill` / glowing 3D blue Liquid Glass selection capsule (matching `X: 350.0` in `SelectionInfo` mockup) and participates in `Tab` / `Shift+Tab` focus chain. | Uses `2.0pt` `-[NSTextFieldCell drawingRectForBounds:]` horizontal inset on compact frames (`height < 21pt` or `width <= 54pt`, `REQ-013-08.2`) for 0% text truncation; receives flowing focus capsule from previously focused field via `DualSpringMorphBlob`. |
| **23** | **`MacSearchField`** | Pill-shaped recessed glass search bar with left magnifying-glass icon, placeholder text, and right clear (`⊗`) button when non-empty. | Filters bound scene/collection/command-palette nodes in real time. | Expands outer focus ring with `CASpring` on focus; clear button fades in/out smoothly as query length changes. |
| **24** | **`MacTokenField`** | Recessed multi-token input well rendering committed items as individual 3D Liquid Glass removable pills (`×`) alongside a live trailing text-edit cursor. | Emits token array mutations on comma/`Enter`/`Backspace`. | Individual token pills lift and highlight their Fresnel rim when hovered or selected. |
| **25** | **`MacTextView`** | Multi-line scrollable recessed glass document editor with optical typography tracking and selection highlight. | Emits multi-line text buffer changes and scroll telemetry to parent `MacScrollView`. | Focus ring animates around outer scroll frame; adapts text ink to paper/well surface luminance. |
| **26** | **`MacLabel`** | Resolution-independent typographic label (`Regular`, `Medium`, `Semibold`, `Bold`, `Mono`) with Apple SF Pro optical tracking (`sf_pro_tracking`) and baseline anchoring. | Pure content ink on glass (emits zero bezel clutter); optionally forwards click to paired control. | Queries `theme.active_surface_at(rect.center())` (`SPEC-006`/`002A`) to dynamically compute WCAG 2.1 AA ($\ge 4.5:1$) foreground contrast over any background or Liquid Glass sheet. |
| **27** | **`MacOriginMatrix`** | $3 \times 3$ grid of interconnected anchor nodes (top-left through bottom-right + center) for bounding-box transform origin selection. | Emits active [`MacOriginAnchor`] and updates transform matrix pivot in host canvas. | Active anchor node glows as a 3D accent LED dot with specular pinpoint; hovered node pre-illuminates along the connecting grid hairlines. |
| **28** | **`MacTagColorPicker`** | Single- or multi-row matrix of 7 macOS Finder semantic color dots (`Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Purple`, `Gray`). | Emits selected tag color index/mask. | Each color dot renders as a 3D glass sphere with upper-left specular dot and expands radially via `CASpring` on hover/selection. |
| **29** | **`MacRuleEditor`** | Multi-row predicate builder (`Any`/`All` header + dynamic criteria rows of `MacPopUpButton`s and `+`/`-` inline action buttons). | Emits predicate row additions, deletions, and criteria changes. | New predicate rows slide and fade into place via `CASpring`; hovered row receives subtle translucent glass highlight band. |
| **30** | **`MacDatePicker`** | Compact stepper-backed date/time field or expanded calendar matrix + analog clock face (`MacDatePickerStyle`). | Emits [`MacDate`] year/month/day/hour/minute mutations. | Selected calendar day cell renders as a raised 3D Liquid Glass accent pill that glides between clicked dates. |
| **31** | **`MacPathControl`** | Horizontal breadcrumb bar of clickable folder/node capsules separated by vector chevrons, plus an optional `Choose...` button. | Emits clicked path component index to navigate hierarchy. | Hovered breadcrumb segment elevates into a 3D glass pill via `CASpring`. |
| **32** | **`MacForm`** (`MacFormUi`) | Two-column right-aligned label / left-aligned control inspector grid engine with baseline alignment. | Coordinates row hover and focus-band geometry across child `MacLabel` and `MacInputCell` pairs. | Propagates active container surface color and glides the active row highlight pill vertically from focused row to focused row. |

---

### 3.5 Family 6: Floating Liquid Glass Surfaces, Navigation, Media & Layout Containers (20 Widgets)

| # | Widget | 1. Base Appearance (*Self / Resting State*) | 2. Outward Emission (*Behavior Toward Environment*) | 3. Inward Reception (*Response From Environment*) |
| :--- | :--- | :--- | :--- | :--- |
| **33** | **`MacPopoverSurface`** | Floating translucent Liquid Glass sheet with Dual-Kawase blurred backdrop, Snellius edge refraction bevel, `1.0pt` outer specular rim, and deep ambient elevation shadow. | Registers its rect in `register_surface_rect` and blocks pointer/scroll leak-through to background canvas (`SPEC-006`). | Samples underlying canvas backdrop for blur/refraction and illuminates its perimeter Fresnel rim closest to the pointer. |
| **34** | **`MacPopover`** | Anchor-attached floating [`MacPopoverSurface`] with directional triangular glass arrow (`Top`, `Bottom`, `Left`, `Right`). | Clamps its frame within window bounds and isolates foreground pointer interaction. | Animates open/close scale and opacity from the anchor point via `hud_panel_transition` `CASpring`. |
| **35** | **`MacPanel`** | Floating utility/HUD window surface (such as `SelectionInfo` in the mockup) with optional title bar and translucent glass body. | Registers panel surface luminance for all child inspector rows and casts soft drop shadow over canvas. | Perimeter glass rim responds to pointer proximity; supports drag-repositioning. |
| **36** | **`MacCard`** | Rounded elevated content card surface with subtle inner top specular hairline and border stroke. | Registers card background surface (`register_surface_rect`) for child typography contrast. | Lifts shadow depth slightly when interactive card mode is hovered. |
| **37** | **`MacBox`** | AppKit `NSBox` grouping container with recessed/raised glass fill, optional title header badge, and configurable content insets. | Registers `box_background` in the per-pass geometric surface registry so both nested and flat `.fixed_rect` NIB children resolve WCAG AA contrast (`SPEC-002A`). | Adapts fill translucency and border contrast to `Aqua`, `DarkAqua`, and `Anthracite` appearances. |
| **38** | **`MacFloatingInfoBox`** | Compact non-modal HUD telemetry pill (`SPEC-006 §2.2`) displaying live coordinates, deltas, or status badges over a canvas. | Floats above canvas without stealing keyboard focus from active tool. | Adapts background vibrancy and label contrast to the canvas region directly beneath it (`REQ-016-06`). |
| **39** | **`MacToolbar`** | Top window toolbar surface with translucent Liquid Glass backdrop, traffic-light/caption-control integration, and grouped toolbar items. | Emits scroll-edge separator luminance and propagates `ProximityLightField` across grouped toolbar buttons (`.glassEffectUnion`). | Dynamically increases backdrop blur and bottom border specular contrast when content scrolls beneath the toolbar. |
| **40** | **`MacMenuBar`** | System/window top menu bar (`File`, `Edit`, `View`, ...) with translucent glass bar and active menu highlight pills. | Spawns cascading [`NSMenu`] dropdown sheets and dispatches global keyboard shortcuts. | Slides active menu header highlight pill horizontally as pointer moves across open menus. |
| **41** | **`NSMenu`** (`NSMenuItem`) | Vertical floating Liquid Glass menu sheet with checkmarks, SF Symbol icons, right-aligned shortcut hints, separators, and cascading submenu arrows. | Dispatches selected menu action and closes menu hierarchy. | Glides a rounded 3D accent selection pill vertically from hovered `NSMenuItem` to hovered `NSMenuItem`. |
| **42** | **`MacCommandPalette`** | Centered modal Spotlight-style Liquid Glass HUD (`Cmd+K`) with large search field and fuzzy-ranked result rows. | Executes selected command and dismisses on `Enter`/`Escape`/click-outside. | Enters with `CASpring` scale-pop; active result row highlight pill glides smoothly with `ArrowUp`/`ArrowDown` or pointer hover. |
| **43** | **`MacCanvasFooter`** | Deterministic `24.0pt` bottom status & breadcrumb bar with zoom preset menu and live coordinate telemetry. | Emits [`MacCanvasFooterAction`] zoom/breadcrumb events to host canvas. | Interactive breadcrumb and zoom capsules highlight their glass rims on pointer hover. |
| **44** | **`MacContextMenu`** | Right-click / secondary-click floating Liquid Glass context menu clamped strictly inside viewport bounds (`clamp_menu_rect`). | Consumes background clicks upon dismissal and returns selected action index. | Opens with `CASpring` origin expansion and glides hover selection capsule across items. |
| **45** | **`MacToast`** (`MacToastManager`) | Floating Liquid Glass notification capsule (`Info`, `Success`, `Warning`, `Error`) with semantic icon badge and `2.5s` auto-dismiss lifecycle. | Stacks multiple active toasts with spring-spaced vertical offsets without blocking canvas work. | Slides in and out via `CASpring` physics and pauses auto-dismiss while hovered. |
| **46** | **`MacPlayerView`** | QuickTime / AVKit media viewport with floating Liquid Glass transport HUD pod (`Play/Pause`, scrub bar, timecode, audio/volume, speed menu). | Drives media playback state and auto-hides the floating glass transport pod when cursor is stationary (`REQ-012-01`). | Floating transport pod samples and blurs the live video frame directly beneath it, refracting video colors at the pod's curved glass rim! |
| **47** | **`MacTrimScrubber`** | AVKit waveform timeline bar with named zones, draggable yellow/accent In/Out trim handles, and `MM:SS.d` playhead lens badge. | Emits trim range `[in_sec, out_sec]` and playhead scrub time. | Playhead badge acts as a magnifying glass pill over the waveform; pauses timeline repaints when window is unfocused (`0.0%` background CPU). |
| **48** | **`MacLevelIndicator`** | Continuous or discrete segmented audio/signal meter with green/yellow-warning/red-critical thresholds and peak-hold tick. | Visualizes live dB/signal telemetry with zero heap allocations. | Illuminated segments glow as translucent glass LED bars with top specular sheen. |
| **49** | **`MacProgressIndicator`** | Determinate/indeterminate horizontal glass progress bar (`Bar`) or 12-spoke fin spinner (`Spinning`). | Communicates task progress while automatically pausing animation repaints when window is unfocused or minimized (`SPEC-011` battery invariant). | Filled progress bar renders a travelling liquid specular sheen across the glass tube. |
| **50** | **`MacTableView`** | Multi-column virtualized `NSTableView` + `NSTableHeaderView` with sort chevrons, alternating row fills, and keyboard navigation. | Emits sort column/direction and single/range/multi row selection changes. | Sticky glass header bar blurs scrolling rows passing beneath it; selected rows render with a cohesive rounded glass selection highlight. |
| **51** | **`MacOutlineView`** | Hierarchical tree view (`MacOutlineTree` / `MacOutlineRow`) with animated `CASpring` disclosure chevrons and indentation guides. | Emits node expansion/collapse and selection events. | Hovered and selected tree rows receive smooth Liquid Glass pill highlights inside the sidebar surface. |
| **52** | **`MacCollectionView`** (`MacThumbnailCell`, `MacImageView`, `MacScrollView`, `MacSplitView`, `MacStackView`, `MacFixedView`, `MacRow`) | $O(1)$ row-culled 2D grid of [`MacThumbnailCell`] items (with strict `clipsToBounds = YES` inner preview & `16pt` footer isolation, `REQ-013-08.3`), framed [`MacImageView`] wells, auto-fading [`MacScrollView`] glass scrollbars, [`MacSplitView`] dividers, and deterministic layout containers. | Layouts cull offscreen items, enforce `subpixel_snap_rect` pixel alignment, and propagate container rects to child controls. | Hovered `MacThumbnailCell` lifts with a 3D glass border glow while strictly clipping inner glyph/image previews above the footer bar; `MacScrollView` thumb expands on hover. |

---

## 4. Detailed Normative Mathematics for Block A (`Iter 1: M1`, `Iter 2: M2`, `Iter 3: M3`)

### 4.1 Iteration 1 (`M1`): Analytical 2D/3D SDF Geometry & Pebble-Cushion Topology (`REQ-016-01`)

1. **Exact Rounded Annular Sector SDF (`sdf_annular_rounded_sector`):**
   - Given polar coordinates $(r, \theta)$ relative to `hub_center` and sector center angle $\theta_c$, let $\Delta\theta = |\text{shortest\_angle}(\theta, \theta_c)|$.
   - To form a constant-radius rounded pebble cushion with corner radius $r_c$ (`8.0–10.0 pt`) inside radial bounds $[r_{\text{in}}, r_{\text{out}}]$ and half-angular sweep $\alpha = \text{sweep}/2$:
     - The inner core annular sector is shrunk by $r_c$ on all 4 boundaries: $r_{\text{in}}' = r_{\text{in}} + r_c$, $r_{\text{out}}' = r_{\text{out}} - r_c$, and angular half-span $\alpha'(r) = \max\!\left(0,\; \alpha - \frac{r_c}{\max(r, r_{\text{in}}')}\right)$.
     - Let $d_{\text{core}}(p)$ be the exact Euclidean distance from $p$ to the clamped core point $p_{\text{core}} = (r_{\text{clamped}}, \theta_{\text{clamped}})$ with $r_{\text{clamped}} = \text{clamp}(r, r_{\text{in}}', r_{\text{out}}')$ and $\theta_{\text{clamped}} = \theta_c + \text{clamp}(\theta - \theta_c, -\alpha'(r_{\text{clamped}}), +\alpha'(r_{\text{clamped}}))$.
     - For interior points inside the core sector, $d_{\text{core}}(p) = -\min\!\left(r - r_{\text{in}}',\; r_{\text{out}}' - r,\; r\sin(\alpha'(r) - \Delta\theta)\right)$.
     - The rounded pebble cushion SDF is then:
       $$d_{\text{cushion}}(p) = d_{\text{core}}(p) - r_c$$
     - Every point on the cushion boundary ($d_{\text{cushion}}(p) = 0$) has a guaranteed minimum curvature radius of $r_c$, producing the exact smooth pebble silhouette of the Golden Reference Mockups (`render1_dark_mode_stand1_v3` & `render3_light_mode_aqua_stand1_v2`).
2. **Exact Curved Stadium Arc SDF (`sdf_curved_stadium_arc` for `"↶ Undo"` & `"↷ Redo"`):**
   - Given centerline radius $r_{\text{mid}} = (r_{\text{skirt\_in}} + r_{\text{skirt\_out}})/2$, capsule half-thickness $h_c = (r_{\text{skirt\_out}} - r_{\text{skirt\_in}})/2$, and angular span $[\theta_0, \theta_1]$:
     - Shrink the centerline arc by $\Delta\theta_{\text{cap}} = h_c / r_{\text{mid}}$ at both ends so the semicircular end-caps stay within $[\theta_0, \theta_1]$: $\theta_0' = \theta_0 + \Delta\theta_{\text{cap}}$, $\theta_1' = \theta_1 - \Delta\theta_{\text{cap}}$.
     - For any point $p$, project its angle $\theta$ onto $[\theta_0', \theta_1']$ to obtain the closest centerline point $q(\theta_{\text{clamped}}) = \text{center} + r_{\text{mid}}(\cos\theta_{\text{clamped}}, \sin\theta_{\text{clamped}})$.
     - Then:
       $$d_{\text{stadium}}(p) = \|p - q(\theta_{\text{clamped}})\|_2 - h_c$$
     - This produces true semicircular $180^\circ$ stadium end-caps at both ends of the curved Undo and Redo buttons.
3. **3D Dome Normal & Optical Centroid from SDF:**
   - From any negative interior SDF value $d(p) \le 0$ and numerical/analytical unit gradient $\vec{g}(p) = \nabla d(p) / \|\nabla d(p)\|_2$:
     $$s(p) = \text{clamp}\!\left(\frac{-d(p)}{w_{\text{bevel}}},\; 0.0,\; 1.0\right), \quad \theta(p) = \text{smoothstep}(0, 1, s(p))\cdot\frac{\pi}{2}, \quad \vec{N}(p) = \big(g_x\cos\theta(p),\; g_y\cos\theta(p),\; \sin\theta(p)\big)$$

### 4.2 Iteration 2 (`M2`): Optical Physics — Snellius, Fresnel, Sub-Surface Caustics & Proximity Light Field (`REQ-016-02`)

1. **5-Component Volumetric Glass Vertex Shader (`shade_liquid_glass_vertex`):**
   - Given base tint $C_{\text{base}}$, accent tint $C_{\text{accent}}$, 3D normal $\vec{N}(p) = (n_x, n_y, n_z)$, normalized SDF penetration $s(p) \in [0, 1]$, and pointer-steered light direction $\vec{L}(\vec{p}(t))$:
     1. **Top-to-Bottom Dome Gradient**: Vertical normal modulation $0.5 + 0.5(-n_y)$ brightens the upper hemisphere and deepens the lower hemisphere.
     2. **Radial Sub-Surface Caustic Core**: Interior dome glow proportional to $s(p)^{1.5} \cdot \text{active\_glow}$, giving the active/latched cushion (`1:1+⊙` in the mockup) its deep luminous cyan-blue core while keeping the bevel darker/richer.
     3. **Blinn-Phong Specular Hotspot**: $I_{\text{spec}} = \max(0, \vec{N} \cdot \vec{H})^8$ smoothly shifts across the dome as $\vec{p}(t)$ moves.
     4. **Directional Schlick Fresnel Rim**: Concentrated in the rim zone $(1 - s(p))^3$, boosted on the light/cursor-facing edge.
     5. **Proximity Light Field (`proximity_rim_gain`)**: Even on unhovered neighboring cushions/controls, vertices within Euclidean distance $R_{\text{prox}}$ (`72.0 pt`) of the live cursor receive a smooth cosine-falloff rim brightening.

### 4.3 Iteration 3 (`M3`): Coupled `CASpring` Dynamics (`DualSpringMorphBlob` & `AnisotropicCushionSpring`, `REQ-016-03..04`)

1. **`DualSpringMorphBlob` (Velocity-Stretched Liquid Droplet Transfer):**
   - Models a 1D or polar angular interval $[a(t), b(t)]$ (or 2D capsule rect) using two coupled `CASpring` instances:
     - **Leading edge** (moving toward the new target) uses a fast responsive spring ($T_{\text{lead}} = 0.22\text{s}, \zeta = 0.82$),
     - **Trailing edge** (following behind) uses a slightly softer trailing spring ($T_{\text{trail}} = 0.32\text{s}, \zeta = 0.76$).
   - During transit from segment/sector $A$ to $B$, $|b(t) - a(t)|$ automatically **stretches** wider than its resting width and then **coalesces** back to resting width upon arrival, with `is_at_rest()` returning `true` once both springs settle.

---

## 5. Detailed Normative Specification for Block B, Family 2 (`Iter 6: W2-V` & `Iter 7: W2-C` — `MacEdgeThumbwheel` & `MacFreeJoystickBowl`)

### 5.1 `MacEdgeThumbwheel` (`Component B` in Golden Reference Mockups `render1_dark_mode_stand1_v3` & `render3_light_mode_aqua_stand1_v2`)

1. **1. Base Appearance (*Self / Resting State*):**
   - **Recessed Glass Trough Housing**: Outer rounded glass trough (`outer_cr`) with soft outer cyan/white ambient halo (`rect.expand(gap)`), `1.0pt` specular top rim, and deep recessed inner groove shadow around `inner_rect`.
   - **32-Band 2D-Grid Metallic/Glass Cylindrical Drum Mesh**:
     - The drum surface inside `inner_rect` is tessellated as a 2D cylindrical mesh with vertical angle $\phi(y) \in [-\frac{\pi}{2}, +\frac{\pi}{2}]$ and horizontal axial normalized coordinate $u(x) \in [-1, +1]$.
     - The vertical cylindrical luminance profile features a bright upper-center **specular reflection band** at $\phi_{\text{spec}} \approx -0.22\text{ rad}$ (modulated dynamically by pointer offset $p_y(t)$), deep curvature shadow at $\phi \to \pm\frac{\pi}{2}$, and a secondary lower-hemisphere bounce reflection.
   - **Illuminated 3D Blue/Accent End-Cap Rings**:
     - The left and right cylinder ends (`[inner_rect.min.x .. inner_rect.min.x + cap_w]` and `[inner_rect.max.x - cap_w .. inner_rect.max.x]`) render as 3D curved end-cap bevels with bright vertical specular core and saturated `.glass_accent(Color32)` neon rim illumination.
   - **Foreshortened 3D Tactile Ridges (Shadow + Highlight Pair)**:
     - Each ridge at angle $\phi_k$ renders a dual-line 3D groove (dark micro-trough at $y(\phi_k) + \text{bw}$ paired with a bright specular ridge crest at $y(\phi_k)$) whose thickness and opacity scale with $\cos(\phi_k)$.
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Emits `value_delta`, `delta_ticks`, `is_free_spinning`, and `MacTapticEvent::{RotaryDetentTick, FlywheelBraked}`; when paired above `MacContextThumbDial`, shares the unified `.glass_accent(Color32)` vibrancy palette.
3. **3. Inward Reception (*Response From Environment*):**
   - Tracks pointer vertical position $p_y(t)$ via `CASpring` to shift the cylindrical specular band in real time; responds to slow drags (Ratchet Mode), fast flicks ($\ge 120\text{ pt/s}$, Free-Spin Flywheel inertia $\omega(t) = \omega_0 e^{-\lambda t}$), and immediate **Touch-to-Brake** on finger contact.

### 5.2 `MacFreeJoystickBowl` (`Stand 2` in Golden Reference Mockups `screen2_dark_mode_stand2_joystick_bowl` & `screen4_light_mode_aqua_stand2_joystick_bowl_v2`)

1. **1. Base Appearance (*Self / Resting State*):**
   - **Single Smooth 3D Outer Glass Bezel & Recessed $360^\circ$ Dish**:
     - Shares the exact single-bezel outer glass rim and semicircular `"↶ Undo"` / `"↷ Redo"` curved-baseline stadium capsules of `MacContextThumbDial` (`r_bowl = r_outer - gap * 2.0`, with zero flat double outer ring).
     - The recessed dish renders a radially shaded concave bowl (`egui::epaint::Mesh` radial rings from deeper inner-top shadow rim to luminous pearl/dark-glass concave floor).
     - Concentric `Fine` ($0.45 R$) and `Fast` ($0.82 R$) rings render as `48` crisp circular micro-dots with `"Fine"` and `"Fast"` labels centered in the lower hemisphere.
   - **3D Volumetric Refractive Crystal Glass Marble (`paint_crystal_glass_marble`)**:
     - Tessellated as a multi-ring 3D spherical `egui::epaint::Mesh` (`puck_r`) rather than flat `circle_filled` primitives:
       1. Multi-layer soft ambient + blue/accent **caustic floor glow** cast onto the bowl floor in the direction away from the top-left light source,
       2. Concave refractive sphere body mesh with Schlick Fresnel rim darkening/brightening around the perimeter ($r \to \text{puck\_r}$),
       3. Internal lower-right **cyan-azure caustic lens focus** inside the marble,
       4. Crisp `1.5pt` outer Fresnel specular ring, and
       5. Upper-left elliptical **Blinn-Phong primary specular highlight** + secondary lower-right internal reflection pinpoint.
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Emits quadratic radial velocity vector `nudge_delta`, `MacTapticEvent::JoystickRingCrossed` when crossing `Fine` or `Fast` rings, and `undo_clicked` / `redo_clicked`.
3. **3. Inward Reception (*Response From Environment*):**
   - When dragged, the crystal marble glides smoothly across $360^\circ$ inside `r_bowl`; when released, instead of teleporting instantaneously to `(0, 0)` in 1 frame, the marble **springs back to the center origin via an underdamped `CASpring`** (`joystick_return_x`, `joystick_return_y`), with `0` idle repaints once settled at center.

---

## 6. Detailed Normative Specification for Block B, Family 3 (`Iter 8: W3-V` & `Iter 9: W3-C` — Morphing Selection Bars: `MacSegmentedControl`, `MacTabView`, `MacTableActionBar`)

1. **1. Base Appearance (*Self / Resting State*):**
   - **Recessed Glass Track & 3D Liquid Selection Droplet (`paint_liquid_glass_capsule`):**
     - The outer track renders a recessed glass groove (`bg_fill` + `separator` inner border).
     - The active segment indicator renders as an elevated 3D Liquid Glass droplet (`paint_liquid_glass_capsule`) with a multi-band vertical dome gradient (`±4%` vertical bezel sheen + top specular rim + bottom ambient drop shadow).
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Emits `SegmentEvent`, active tab index, and registers the elevated pill surface so child text/icons automatically adapt their vibrancy contrast via `theme.label_on(pill_surface)`.
3. **3. Inward Reception (*Response From Environment* — `DualSpringMorphBlob`):**
   - When the active selection index changes from $i$ to $j$, the active capsule transitions via `step_dual_spring_morph_interval(ctx, id, target_min_x, target_max_x, theme)`:
     - When moving right ($\Delta x > 0$), the right/leading edge uses the faster snappy spring ($T_{\text{lead}} = 0.5 T_0$) while the left/trailing edge uses the standard spring ($T_{\text{trail}} = T_0$).
     - When moving left ($\Delta x < 0$), the left/leading edge uses $T_{\text{lead}}$ and the right/trailing edge uses $T_{\text{trail}}$.
     - Consequently, during mid-flight transit ($t \in [0.02\text{s}, 0.12\text{s}]$), the selection capsule **stretches wider than its resting width** like a physical liquid droplet before coalescing smoothly onto the destination segment with `0` idle repaints once settled.

---

## 7. Detailed Normative Specification for Block B, Family 4 (`Iter 10: W4-V` & `Iter 11: W4-C` — Action & Value Primitives + `BUG-SONOMA-01` & `BUG-SONOMA-03`)

1. **1. Base Appearance (*Self / Resting State*) & `BUG-SONOMA-03` Fix:**
   - **`paint_liquid_glass_capsule` (`±4%` Vertical Dome Bezel Sheen + Top Specular Rim):**
     - `MacButtonCell`, `MacPopUpButton` (bezel & accent badge), and `MacToggle` render with a genuine 3D convex bezel sheen (`paint_liquid_glass_capsule`) rather than a single flat solid `painter.rect_filled` call (`BUG-SONOMA-03`), while strictly preserving `LayoutPolicy::ExactAppleNib` frame coordinates (`<= 0.5pt` invariant, `REQ-016-07`).
     - `MacSlider` renders its circular or pill thumb as a 3D convex Liquid Glass droplet with top specular rim, ambient drop shadow, and `CASpring` press scale.
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Emits value mutations, click/toggle responses, and adaptive `MacBackgroundStyle` contrast for inner labels and chevrons.
3. **3. Inward Reception (*Response From Environment*) & `BUG-SONOMA-01` Fix (`effective_control_accent`):**
   - **Window-Active Accent De-saturation (`BUG-SONOMA-01`, `MacTheme::effective_control_accent`):**
     - When the host window is active (`is_window_active(ui.ctx()) == true`), accented controls (`MacButtonCell::selected(true)`, `MacCheckbox`, `MacRadioButton`, `MacSlider` active track, `MacPopUpButton::accent_badge(true)`) render with full `theme.control_accent()` / `theme.selected_control()`.
     - When the host window becomes inactive/unfocused (`is_window_active(ui.ctx()) == false`), `theme.effective_control_accent(ui.ctx())` automatically de-saturates to neutral graphite (`theme.selected_control_inactive()`), matching native macOS Sonoma / Tahoe window deactivation semantics.

---

## 8. Detailed Normative Specification for Block B, Family 5 (`Iter 12: W5-V` & `Iter 13: W5-C` — Text, Inspector & Form Controls + `BUG-SONOMA-02`)

1. **1. Base Appearance (*Self / Resting State*):**
   - `MacInputCell` preserves the exact `2.0pt` `-[NSTextFieldCell drawingRectForBounds:]` horizontal inset and `0%` text truncation guarantee across all NIB dimensions (`REQ-013-08.2`), with a `1.0pt` top inner shadow groove (`input_top_inset_shadow`).
   - Supports `.liquid_glass_pill(true)` (matching the glowing 3D blue `X: 350.0` active row in the `SelectionInfo` inspector panel of the golden reference mockups).
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Emits transactional text/numeric mutations and participates in cyclic `Tab` / `Shift+Tab` focus transfer.
3. **3. Inward Reception (*Response From Environment*) & `BUG-SONOMA-02` Fix (`paint_animated_focus_ring`):**
   - **Spring-Animated Focus Ring Halo (`BUG-SONOMA-02`):**
     - Instead of popping in instantaneously in 1 frame, `MacTheme::paint_animated_focus_ring(ui.ctx(), id, &painter, rect, corner_radius, is_focused)` animates the focus ring expansion and opacity via a fast `CASpring` ($T = 0.5 T_{\text{snappy}} \approx 0.12\text{s}$), contracting smoothly when focus is surrendered and requesting `0` repaints at rest.

---

## 9. Detailed Normative Specification for Block B, Family 6 (`Iter 14: W6-V` & `Iter 15: W6-C` — Floating Surfaces, Navigation & Media: `MacPopoverSurface`, `MacPopover`, `MacPanel`, `MacFloatingInfoBox`)

1. **1. Base Appearance (*Self / Resting State*):**
   - Floating glass surfaces (`MacPopoverSurface`, `MacPopover`) render a 3D Liquid Glass material layer combining `popover_background`, subtle top-to-bottom convex glass sheen, and a crisp `1.0pt` `specular_rim_top` inner highlight along the top edge.
2. **2. Outward Emission (*Behavior Toward Environment*):**
   - Registers its surface vibrancy slot via `theme.register_surface_rect_slot(ui.ctx(), rect, bg_fill)` so all nested controls inside the popover/panel automatically adapt their foreground contrast and vibrancy.
3. **3. Inward Reception (*Response From Environment*):**
   - Samples and refracts any registered background canvas or underlying window surface through the Liquid Glass optical pipeline (`build_liquid_glass_backdrop_image`, Dual-Kawase blur + $1.5\times$ saturation boost + Snellius perimeter refraction).


