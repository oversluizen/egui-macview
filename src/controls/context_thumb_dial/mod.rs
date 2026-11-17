//! AppKit "Liquid Glass" 2-Layer Context Thumb Dial (`MacContextThumbDial`, `SPEC-014`).
//!
//! Provides a compact, bimanual radial controller with two center-toggled layers:
//! - **Layer 1 (`MacThumbDialLayer::CompassModifiers`)**: Six frosted-glass radial cushions
//!   (`North`, `NorthEast`, `East`, `SouthEast`, `South`, `West`) with live tool labels,
//!   *Tap-to-Latch* (`< 180 ms`), and *Hold-to-Momentary* (`>= 180 ms`).
//! - **Layer 2 (`MacThumbDialLayer::FreeJoystickBowl`)**: An unobstructed $360^\circ$ analog
//!   joystick bowl with `Fine` and `Fast` concentric speed rings and quadratic radial velocity scaling.

mod render;

use crate::theme::MacTheme;
use crate::tokens::ThumbDialMetrics;
use egui::{Id, Pos2, Rect, Response, Ui, Vec2};
use std::hash::Hash;

/// Active operating layer of [`MacContextThumbDial`], toggled by tapping the center button (`REQ-014-02`, `REQ-014-03`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum MacThumbDialLayer {
    /// Layer 1: 6-sector compass modifier cushions (`N`, `NE`, `E`, `SE`, `S`, `W`).
    #[default]
    CompassModifiers,
    /// Layer 2: Unobstructed $360^\circ$ analog point-nudge joystick bowl (`Fine` inner -> `Fast` outer).
    FreeJoystickBowl,
}

impl MacThumbDialLayer {
    /// Toggles between `CompassModifiers` (Layer 1) and `FreeJoystickBowl` (Layer 2).
    #[inline]
    pub const fn toggle(self) -> Self {
        match self {
            Self::CompassModifiers => Self::FreeJoystickBowl,
            Self::FreeJoystickBowl => Self::CompassModifiers,
        }
    }
}

/// Placement of the curved `↶ Undo` and `↷ Redo` crescent buttons around the dial (`REQ-014-05`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum MacThumbArcPlacement {
    /// Both `↶ Undo` (top-left) and `↷ Redo` (top-right) sit as a crown above the circle (zero thumb occlusion).
    #[default]
    TopCrown,
    /// Both `↶ Undo` (bottom-left) and `↷ Redo` (bottom-right) sit along the bottom skirt.
    BottomSkirt,
    /// `↶ Undo` sits as a top cap (`12:00`) and `↷ Redo` sits as a bottom cap (`6:00`).
    TopAndBottomCaps,
}

/// Universal geometric modifier intentions mapped to the 4 cardinal compass directions (`REQ-014-01`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacThumbIntent {
    /// North (`⇧` / `Shift`): Constrain angle ($45^\circ / 15^\circ$) or lock $1:1$ aspect ratio.
    Constrain = 0,
    /// East (`⌥` / `Alt`): Alter topology (break cusp handle $\wedge$, clone $\⧉$, center origin $\odot$, add nodes $\oplus$).
    Alter = 1,
    /// South (`⌘` / `Ctrl`): Glide / Servo along curve ($\leadsto$), $G^2$ Genius ($\star$), shear ($\rightleftarrows$), or snap ($\⌖$).
    Glide = 2,
    /// West (`␣` / `Space`): Move in-flight anchor point ($\✥$) or pan canvas ($\✋$).
    MovePan = 3,
}

impl MacThumbIntent {
    /// Bitflag representation (`1 << variant`).
    #[inline]
    pub const fn bit(self) -> u8 {
        1u8 << (self as u8)
    }
}

/// Bitmask of active [`MacThumbIntent`] flags (supporting multi-sector chording).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct MacThumbIntentMask(pub u8);

impl MacThumbIntentMask {
    /// Empty intent mask.
    pub const NONE: Self = Self(0);
    /// North (`⇧ Constrain`).
    pub const CONSTRAIN: Self = Self(1 << 0);
    /// East (`⌥ Alter / Break`).
    pub const ALTER: Self = Self(1 << 1);
    /// South (`⌘ Glide / Servo`).
    pub const GLIDE: Self = Self(1 << 2);
    /// West (`␣ Move / Pan`).
    pub const MOVE_PAN: Self = Self(1 << 3);

    /// Returns `true` if `intent` is set in this mask.
    #[inline]
    pub const fn contains(self, intent: MacThumbIntent) -> bool {
        (self.0 & intent.bit()) != 0
    }

    /// Returns `true` if all bits in `other` are present in `self` (and `other` is non-empty).
    #[inline]
    pub const fn contains_mask(self, other: Self) -> bool {
        other.0 != 0 && (self.0 & other.0) == other.0
    }

    /// Returns the bitwise union of `self` and `other`.
    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Toggles all bits of `other` inside `self`.
    #[inline]
    pub const fn toggle_mask(self, other: Self) -> Self {
        if self.contains_mask(other) {
            Self(self.0 & !other.0)
        } else {
            Self(self.0 | other.0)
        }
    }

    /// Returns `true` when no intent bits are active.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Compass sectors of Layer 1 (`REQ-014-01`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacThumbSector {
    /// Top cardinal sector (`-90°`, `⇧ Constrain`).
    North = 0,
    /// Upper-right diagonal combination sector (`-45°`, `⇧+⌥ Constrain + Alter`).
    NorthEast = 1,
    /// Right cardinal sector (`0°`, `⌥ Alter / Break`).
    East = 2,
    /// Lower-right diagonal combination sector (`+45°`, `⌘+⌥ Glide + Alter`).
    SouthEast = 3,
    /// Bottom cardinal sector (`+90°`, `⌘ Glide / Servo`).
    South = 4,
    /// Left cardinal sector (`180°`, `␣ Move / Pan`).
    West = 5,
}

impl MacThumbSector {
    /// Ordered list of all 6 clover sectors.
    pub const ALL: [Self; 6] = [Self::North, Self::NorthEast, Self::East, Self::SouthEast, Self::South, Self::West];

    /// Returns the semantic [`MacThumbIntentMask`] activated by this sector.
    #[inline]
    pub const fn intent_mask(self) -> MacThumbIntentMask {
        match self {
            Self::North => MacThumbIntentMask::CONSTRAIN,
            Self::NorthEast => MacThumbIntentMask::CONSTRAIN.union(MacThumbIntentMask::ALTER),
            Self::East => MacThumbIntentMask::ALTER,
            Self::SouthEast => MacThumbIntentMask::GLIDE.union(MacThumbIntentMask::ALTER),
            Self::South => MacThumbIntentMask::GLIDE,
            Self::West => MacThumbIntentMask::MOVE_PAN,
        }
    }

    /// Returns `true` if this sector is a diagonal combination wedge (`NorthEast` or `SouthEast`).
    #[inline]
    pub const fn is_diagonal_combo(self) -> bool {
        matches!(self, Self::NorthEast | Self::SouthEast)
    }

    /// Returns the center angle in degrees from `ThumbDialMetrics`.
    pub fn center_angle_deg(self, m: &ThumbDialMetrics) -> f32 {
        match self {
            Self::North => m.angle_north_deg.points(),
            Self::NorthEast => m.angle_north_east_deg.points(),
            Self::East => m.angle_east_deg.points(),
            Self::SouthEast => m.angle_south_east_deg.points(),
            Self::South => m.angle_south_deg.points(),
            Self::West => m.angle_west_deg.points(),
        }
    }

    /// Returns the angular sweep in degrees from `ThumbDialMetrics`.
    pub fn sweep_deg(self, m: &ThumbDialMetrics) -> f32 {
        if self.is_diagonal_combo() {
            m.diagonal_sweep_deg.points()
        } else {
            m.cardinal_sweep_deg.points()
        }
    }
}

/// Visual label and desktop shortcut hint for a single [`MacThumbSector`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacThumbSectorSpec<'a> {
    /// Live semantic action label (e.g. `"∧ Cusp"`, `"1:1"`, `"⊙ Center"`, `"↝ Slide"`).
    pub label: &'a str,
    /// Subtle desktop shortcut hint (e.g. `"⇧"`, `"⌥"`, `"⌘"`, `"␣"`, `"⇧+⌥"`, `"⌘+⌥"`).
    pub shortcut_hint: &'a str,
}

/// Complete 6-sector label table for [`MacContextThumbDial`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacThumbSectorSet<'a> {
    /// Sector descriptors indexed in [`MacThumbSector::ALL`] order.
    pub items: [MacThumbSectorSpec<'a>; 6],
}

impl<'a> MacThumbSectorSet<'a> {
    /// Returns the descriptor for `sector`.
    #[inline]
    pub const fn sector(&self, sector: MacThumbSector) -> MacThumbSectorSpec<'a> {
        self.items[sector as usize]
    }
}

/// Built-in tool context presets from `SPEC-014 §2.1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacThumbContextPreset {
    /// Cubic Bézier & Spline Pen tool (`P`).
    Pen,
    /// Node & Contour Select tool (`V`).
    Select,
    /// Geometric Primitives (`F`) & Bounding-Box Transform tool.
    #[default]
    Primitives,
    /// Parametric Knife slicing tool (`E`).
    Knife,
}

impl MacThumbContextPreset {
    /// All 4 canonical tool presets.
    pub const ALL: [Self; 4] = [Self::Pen, Self::Select, Self::Primitives, Self::Knife];

    /// Display name of the tool preset.
    pub const fn title(self) -> &'static str {
        match self {
            Self::Pen => "Pen (P)",
            Self::Select => "Select (V)",
            Self::Primitives => "Primitives / BBox (F)",
            Self::Knife => "Knife (E)",
        }
    }

    /// Resolves the 6-sector semantic label and desktop hint table for this preset (`REQ-014-01`).
    pub const fn sectors(self) -> MacThumbSectorSet<'static> {
        match self {
            Self::Pen => MacThumbSectorSet {
                items: [
                    MacThumbSectorSpec { label: "∠ 45°", shortcut_hint: "⇧" },
                    MacThumbSectorSpec { label: "∠45°+∧Cusp", shortcut_hint: "⇧+⌥" },
                    MacThumbSectorSpec { label: "∧ Cusp", shortcut_hint: "⌥" },
                    MacThumbSectorSpec { label: "★G2+∧Cusp", shortcut_hint: "⌘+⌥" },
                    MacThumbSectorSpec { label: "★ Genius", shortcut_hint: "⌘" },
                    MacThumbSectorSpec { label: "✥ Move Pt", shortcut_hint: "␣" },
                ],
            },
            Self::Select => MacThumbSectorSet {
                items: [
                    MacThumbSectorSpec { label: "∠ Axis", shortcut_hint: "⇧" },
                    MacThumbSectorSpec { label: "∠Axis+⧉Clone", shortcut_hint: "⇧+⌥" },
                    MacThumbSectorSpec { label: "⧉ Clone", shortcut_hint: "⌥" },
                    MacThumbSectorSpec { label: "↝Slide+1-Side", shortcut_hint: "⌘+⌥" },
                    MacThumbSectorSpec { label: "↝ Slide", shortcut_hint: "⌘" },
                    MacThumbSectorSpec { label: "✋ Pan", shortcut_hint: "␣" },
                ],
            },
            Self::Primitives => MacThumbSectorSet {
                items: [
                    MacThumbSectorSpec { label: "1:1", shortcut_hint: "⇧" },
                    MacThumbSectorSpec { label: "1:1+⊙Center", shortcut_hint: "⇧+⌥" },
                    MacThumbSectorSpec { label: "⊙ Center", shortcut_hint: "⌥" },
                    MacThumbSectorSpec { label: "⇄ Shear", shortcut_hint: "⌘+⌥" },
                    MacThumbSectorSpec { label: "⇄ Shear", shortcut_hint: "⌘" },
                    MacThumbSectorSpec { label: "✥ Move", shortcut_hint: "␣" },
                ],
            },
            Self::Knife => MacThumbSectorSet {
                items: [
                    MacThumbSectorSpec { label: "∠ 15°", shortcut_hint: "⇧" },
                    MacThumbSectorSpec { label: "∠15°+⊕Nodes", shortcut_hint: "⇧+⌥" },
                    MacThumbSectorSpec { label: "⊕ Nodes", shortcut_hint: "⌥" },
                    MacThumbSectorSpec { label: "⌖Snap+⊕Nodes", shortcut_hint: "⌘+⌥" },
                    MacThumbSectorSpec { label: "⌖ Snap", shortcut_hint: "⌘" },
                    MacThumbSectorSpec { label: "✋ Pan", shortcut_hint: "␣" },
                ],
            },
        }
    }
}

/// Ergonomic thumb-zone docking modes (`REQ-014-05`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacThumbDockZone {
    /// Inline flow allocation inside the current `egui::Ui`.
    #[default]
    Inline,
    /// Anchored in the bottom-left thumb corner of the target canvas rect.
    BottomLeft,
    /// Anchored in the bottom-right thumb corner of the target canvas rect.
    BottomRight,
}

#[derive(Clone, Copy, Default)]
pub(super) struct ThumbDialPressMemory {
    pub press_start_time: Option<f64>,
    pub started_in_hub: bool,
    pub started_in_rotary: bool,
    pub initial_sector: Option<MacThumbSector>,
    pub active_sector: Option<MacThumbSector>,
    pub last_rotary_angle_deg: Option<f32>,
    pub joystick_offset: Vec2,
    pub stored_layer: MacThumbDialLayer,
}

/// Interaction output returned by [`MacContextThumbDial::show`].
#[derive(Debug, Clone)]
pub struct MacContextThumbResponse {
    /// Underlying `egui::Response` of the dial surface.
    pub response: Response,
    /// Active dial layer (`CompassModifiers` or `FreeJoystickBowl`).
    pub active_layer: MacThumbDialLayer,
    /// Sticky latched intent mask (`Tap-to-Latch`).
    pub latched_mask: MacThumbIntentMask,
    /// Momentary held/slid intent mask (`Hold-to-Momentary` or `Center Slide-Out`).
    pub momentary_mask: MacThumbIntentMask,
    /// Effective combined intent mask (`latched_mask | momentary_mask`).
    pub effective_mask: MacThumbIntentMask,
    /// Currently hovered or actively held clover sector.
    pub active_sector: Option<MacThumbSector>,
    /// Rotary encoder tick delta generated on this frame.
    pub rotary_delta_ticks: f32,
    /// Continuous $360^\circ$ point-nudge vector (`(dx, dy)`) generated in Layer 2 (`FreeJoystickBowl`) or via rotary nudge.
    pub nudge_delta: Vec2,
    /// `true` if the curved `↶ Undo` crescent button was clicked.
    pub undo_clicked: bool,
    /// `true` if the curved `↷ Redo` crescent button was clicked.
    pub redo_clicked: bool,
    /// Screen-space center of the central Thumb Rest Hub.
    pub hub_center: Pos2,
    /// Screen-space hit rect of the curved `↶ Undo` button.
    pub undo_rect: Rect,
    /// Screen-space hit rect of the curved `↷ Redo` button.
    pub redo_rect: Rect,
    pub(super) sector_centers: [Pos2; 6],
}

impl MacContextThumbResponse {
    /// Returns the screen-space centroid of `sector` (for hit testing and visual inspection).
    #[inline]
    pub fn sector_center(&self, sector: MacThumbSector) -> Pos2 {
        self.sector_centers[sector as usize]
    }
}

/// 2-Layer AppKit "Liquid Glass" Context-Adaptive Thumb Dial (`SPEC-014`).
///
/// # Example
/// ```rust
/// use egui_macview::{
///     MacAppearance, MacContextThumbDial, MacTheme, MacThumbContextPreset, MacThumbIntentMask,
/// };
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::DarkAqua);
/// let mut latched = MacThumbIntentMask::NONE;
/// let mut rotary_val = 0.5f32;
///
/// let resp = MacContextThumbDial::new("studio_thumb_dial", &mut latched, &mut rotary_val)
///     .preset(MacThumbContextPreset::Primitives)
///     .theme(&theme)
///     .show(ui);
/// # let _ = resp.effective_mask;
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacContextThumbDial<'a> {
    pub(super) id_source: Id,
    pub(super) latched_mask: &'a mut MacThumbIntentMask,
    pub(super) rotary_value: &'a mut f32,
    pub(super) layer_binding: Option<&'a mut MacThumbDialLayer>,
    pub(super) arc_placement: MacThumbArcPlacement,
    pub(super) sectors: MacThumbSectorSet<'a>,
    pub(super) rotary_param_name: &'a str,
    pub(super) rotary_formatted_value: Option<&'a str>,
    pub(super) can_undo: bool,
    pub(super) can_redo: bool,
    pub(super) dock_target: Option<(Rect, MacThumbDockZone)>,
    pub(super) scale: f32,
    pub(super) theme: Option<&'a MacTheme>,
}

impl<'a> MacContextThumbDial<'a> {
    /// Creates a new `MacContextThumbDial` bound to `latched_mask` and `rotary_value` (in `[0.0, 1.0]`).
    pub fn new(id_source: impl Hash + std::fmt::Debug, latched_mask: &'a mut MacThumbIntentMask, rotary_value: &'a mut f32) -> Self {
        Self {
            id_source: Id::NULL.with(id_source),
            latched_mask,
            rotary_value,
            layer_binding: None,
            arc_placement: MacThumbArcPlacement::TopCrown,
            sectors: MacThumbContextPreset::Primitives.sectors(),
            rotary_param_name: "✥ Mode",
            rotary_formatted_value: None,
            can_undo: true,
            can_redo: true,
            dock_target: None,
            scale: 1.0,
            theme: None,
        }
    }

    /// Binds the active 2-layer state (`CompassModifiers` vs. `FreeJoystickBowl`), toggled by clicking the center button.
    pub fn layer(mut self, layer: &'a mut MacThumbDialLayer) -> Self {
        self.layer_binding = Some(layer);
        self
    }

    /// Configures the placement of the curved `↶ Undo` and `↷ Redo` crescent buttons (defaults to [`MacThumbArcPlacement::TopCrown`]).
    pub fn undo_arc_placement(mut self, placement: MacThumbArcPlacement) -> Self {
        self.arc_placement = placement;
        self
    }

    /// Selects one of the canonical tool presets (`Pen`, `Select`, `Primitives`, `Knife`).
    pub fn preset(mut self, preset: MacThumbContextPreset) -> Self {
        self.sectors = preset.sectors();
        self
    }

    /// Injects custom 6-sector labels and shortcut hints (e.g. from `grafia::ControllerTelemetryDescriptor`).
    pub fn custom_sectors(mut self, sectors: MacThumbSectorSet<'a>) -> Self {
        self.sectors = sectors;
        self
    }

    /// Sets the parameter name and formatted readout displayed inside the central hub.
    pub fn rotary_parameter(mut self, name: &'a str, formatted_value: &'a str) -> Self {
        self.rotary_param_name = name;
        self.rotary_formatted_value = Some(formatted_value);
        self
    }

    /// Enables or disables the curved `↶ Undo` button.
    pub fn can_undo(mut self, can_undo: bool) -> Self {
        self.can_undo = can_undo;
        self
    }

    /// Enables or disables the curved `↷ Redo` button.
    pub fn can_redo(mut self, can_redo: bool) -> Self {
        self.can_redo = can_redo;
        self
    }

    /// Docks the dial in the `BottomLeft` or `BottomRight` thumb zone of `canvas_rect`.
    pub fn dock_in_rect(mut self, canvas_rect: Rect, zone: MacThumbDockZone) -> Self {
        self.dock_target = Some((canvas_rect, zone));
        self
    }

    /// Scales the overall radial dimensions (`1.0` default).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.5);
        self
    }

    /// Injects a [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the Liquid Glass 2-layer thumb dial and processes pointer/touch gestures.
    pub fn show(self, ui: &mut Ui) -> MacContextThumbResponse {
        render::show_dial(self, ui)
    }
}
