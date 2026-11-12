//! Authentic macOS AppKit / Pro-Inspector `3x3` Transform Origin / Anchor Pin Grid (`MacOriginMatrix`).

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Nine-point `3x3` transform origin / anchor pin position.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MacOriginAnchor {
    /// Top-left pin (`row = 0, col = 0`).
    TopLeft = 0,
    /// Top-center pin (`row = 0, col = 1`).
    TopCenter = 1,
    /// Top-right pin (`row = 0, col = 2`).
    TopRight = 2,
    /// Middle-left pin (`row = 1, col = 0`).
    MiddleLeft = 3,
    /// Center origin pin (`row = 1, col = 1`).
    #[default]
    Center = 4,
    /// Middle-right pin (`row = 1, col = 2`).
    MiddleRight = 5,
    /// Bottom-left pin (`row = 2, col = 0`).
    BottomLeft = 6,
    /// Bottom-center pin (`row = 2, col = 1`).
    BottomCenter = 7,
    /// Bottom-right pin (`row = 2, col = 2`).
    BottomRight = 8,
}

impl MacOriginAnchor {
    /// All nine anchor positions in row-major order (`0..9`).
    pub const ALL: [Self; 9] = [
        Self::TopLeft, Self::TopCenter, Self::TopRight,
        Self::MiddleLeft, Self::Center, Self::MiddleRight,
        Self::BottomLeft, Self::BottomCenter, Self::BottomRight,
    ];

    /// Returns the row-major index in `0..9`.
    pub const fn index(self) -> usize { self as usize }

    /// Converts a row-major index `0..9` into a [`MacOriginAnchor`], falling back to [`Self::Center`].
    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopCenter,
            2 => Self::TopRight,
            3 => Self::MiddleLeft,
            4 => Self::Center,
            5 => Self::MiddleRight,
            6 => Self::BottomLeft,
            7 => Self::BottomCenter,
            8 => Self::BottomRight,
            _ => Self::Center,
        }
    }

    /// Returns the grid row index in `0..=2`.
    pub const fn row(self) -> usize { (self as usize) / 3 }

    /// Returns the grid column index in `0..=2`.
    pub const fn col(self) -> usize { (self as usize) % 3 }

    /// Constructs a [`MacOriginAnchor`] from `(row, col)`, clamping each axis to `0..=2`.
    pub const fn from_row_col(row: usize, col: usize) -> Self {
        let r = if row > 2 { 2 } else { row };
        let c = if col > 2 { 2 } else { col };
        Self::from_index(r * 3 + c)
    }
}

enum OriginBinding<'a> {
    Anchor(&'a mut MacOriginAnchor),
    Index(&'a mut usize),
}

impl OriginBinding<'_> {
    fn get(&self) -> MacOriginAnchor {
        match self {
            Self::Anchor(a) => **a,
            Self::Index(idx) => MacOriginAnchor::from_index(**idx),
        }
    }

    fn set(&mut self, anchor: MacOriginAnchor) {
        match self {
            Self::Anchor(a) => **a = anchor,
            Self::Index(idx) => **idx = anchor.index(),
        }
    }
}

/// Authentic macOS Pro-Inspector `3x3` Transform Origin / Anchor Pin Grid (`MacOriginMatrix`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacOriginAnchor, MacOriginMatrix, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut anchor = MacOriginAnchor::Center;
/// MacOriginMatrix::new(&mut anchor).theme(&theme).show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacOriginMatrix<'a> {
    binding: OriginBinding<'a>,
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacOriginMatrix<'a> {
    /// Creates a new `3x3` origin matrix bound to a [`MacOriginAnchor`].
    pub fn new(selected: &'a mut MacOriginAnchor) -> Self {
        Self {
            binding: OriginBinding::Anchor(selected),
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Creates a new `3x3` origin matrix bound to a row-major `usize` index (`0..9`).
    pub fn from_index(selected_index: &'a mut usize) -> Self {
        Self {
            binding: OriginBinding::Index(selected_index),
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets the AppKit control size (`Regular` `34pt`, `Small` `28pt`, `Mini` `22pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the origin matrix to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Computes the exact center position of `anchor` inside `rect` given `cell_step`.
    pub fn node_center(rect: Rect, anchor: MacOriginAnchor, cell_step: f32) -> Pos2 {
        let c = rect.center();
        let dx = (anchor.col() as f32 - 1.0) * cell_step;
        let dy = (anchor.row() as f32 - 1.0) * cell_step;
        Pos2::new(c.x + dx, c.y + dy)
    }

    /// Renders the origin matrix into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for MacOriginMatrix<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let om = theme.origin_matrix();

        let (grid_sz, cell_step) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (om.grid_size.points(), om.cell_step.points()),
            MacControlSize::Small | MacControlSize::CompactInfobox => (om.small_grid_size.points(), om.small_cell_step.points()),
            MacControlSize::Mini => (om.mini_grid_size.points(), om.mini_cell_step.points()),
        };

        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_origin_matrix_fixed");
            ui.skip_ahead_auto_ids(1);
            (snapped, ui.interact(snapped, id, sense))
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(grid_sz), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        let mut active = self.binding.get();
        self.binding.set(active);
        if enabled && response.clicked() {
            let click_pos = response.interact_pointer_pos().unwrap_or(rect.center());
            let (mut best, mut best_dist) = (active, f32::INFINITY);
            for candidate in MacOriginAnchor::ALL {
                let d = Self::node_center(rect, candidate, cell_step).distance(click_pos);
                if d < best_dist {
                    best_dist = d;
                    best = candidate;
                }
            }
            if best_dist <= cell_step {
                let changed = best != active;
                active = best;
                self.binding.set(active);
                response.request_focus();
                if changed {
                    response.mark_changed();
                }
            }
        }

        if enabled && response.has_focus() {
            let (mut r, mut c) = (active.row(), active.col());
            ui.input(|i| {
                if i.key_pressed(Key::ArrowLeft) { c = c.saturating_sub(1); }
                if i.key_pressed(Key::ArrowRight) { c = (c + 1).min(2); }
                if i.key_pressed(Key::ArrowUp) { r = r.saturating_sub(1); }
                if i.key_pressed(Key::ArrowDown) { r = (r + 1).min(2); }
            });
            let next = MacOriginAnchor::from_row_col(r, c);
            if next != active {
                active = next;
                self.binding.set(active);
                response.mark_changed();
            }
        }

        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let visual_rect = theme.clamp_nib_visual_rect(rect, grid_sz, ppp);
        if enabled && response.has_focus() {
            theme.paint_focus_ring(&painter, visual_rect, om.node_radius.points());
        }

        let surface = theme.active_surface_at(ui, rect.center());
        let line_col = theme.tertiary_label_on(surface).gamma_multiply(alpha);
        let line_stroke = Stroke::new(om.line_stroke_width.points(), line_col);
        for edge in [0usize, 2usize] {
            let r_start = Self::node_center(rect, MacOriginAnchor::from_row_col(edge, 0), cell_step);
            let r_end = Self::node_center(rect, MacOriginAnchor::from_row_col(edge, 2), cell_step);
            painter.line_segment([r_start, r_end], line_stroke);
            let c_start = Self::node_center(rect, MacOriginAnchor::from_row_col(0, edge), cell_step);
            let c_end = Self::node_center(rect, MacOriginAnchor::from_row_col(2, edge), cell_step);
            painter.line_segment([c_start, c_end], line_stroke);
        }

        let hover_pos = ui.input(|i| i.pointer.hover_pos());
        let widget_hovered = enabled && (response.hovered() || hover_pos.is_some_and(|hp| rect.contains(hp)));
        for anchor in MacOriginAnchor::ALL {
            let pos = Self::node_center(rect, anchor, cell_step);
            if anchor == active {
                painter.circle_filled(
                    pos,
                    om.active_node_radius.points(),
                    theme.control_accent().gamma_multiply(alpha),
                );
            } else {
                let hovered = widget_hovered && hover_pos.is_some_and(|hp| hp.distance(pos) <= cell_step * 0.5);
                let rad = if hovered { om.hover_node_radius.points() } else { om.node_radius.points() };
                let fill_col = if hovered { theme.label_on(surface) } else { theme.secondary_label_on(surface) };
                painter.circle_filled(pos, rad, fill_col.gamma_multiply(alpha));
            }
        }
        response
    }
}
