//! Authentic macOS Finder / AppKit Color Label Swatch Picker (`MacTagColorPicker`).

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Authentic macOS Finder / AppKit Color Label Swatch Picker (`MacTagColorPicker`).
///
/// Renders a `1x12` or `2x6` (or custom column count) grid of circular semantic tag color dots
/// plus an optional clear (`None`) swatch.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacTagColorPicker, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut selected_tag: Option<usize> = Some(0);
/// MacTagColorPicker::new(&mut selected_tag).theme(&theme).show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacTagColorPicker<'a> {
    selected: &'a mut Option<usize>,
    columns: usize,
    rows: Option<usize>,
    custom_dot_size: Option<f32>,
    custom_spacing: Option<(f32, f32)>,
    include_clear_swatch: bool,
    palette: Option<&'a [Color32]>,
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTagColorPicker<'a> {
    /// Creates a new tag color swatch picker bound to `selected` (`None` = clear / no tag).
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            columns: 12,
            rows: None,
            custom_dot_size: None,
            custom_spacing: None,
            include_clear_swatch: true,
            palette: None,
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets the number of columns per row (default `12` for `1x12`, or `6` for `2x6`).
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Sets the number of rows in the swatch grid (e.g. `2` for a `2x6` sidebar layout).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(1));
        self
    }

    /// Overrides the swatch dot diameter in points.
    pub fn dot_size(mut self, size: f32) -> Self {
        self.custom_dot_size = Some(size);
        self
    }

    /// Overrides the horizontal (`col_gap`) and vertical (`row_gap`) spacing between swatches.
    pub fn spacing(mut self, col_gap: f32, row_gap: f32) -> Self {
        self.custom_spacing = Some((col_gap, row_gap));
        self
    }

    /// Controls whether a trailing clear (`None`) swatch is appended (default `true`).
    pub fn include_clear_swatch(mut self, include: bool) -> Self {
        self.include_clear_swatch = include;
        self
    }

    /// Overrides the default 11-color semantic tag palette with a custom slice of [`Color32`].
    pub fn palette(mut self, palette: &'a [Color32]) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Sets the AppKit control size (`Regular` `12pt` dots, `Small` `10pt`, `Mini` `8pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the tag color picker to an exact NIB rectangle.
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

    /// Computes the exact center position of swatch `index` inside `rect`.
    pub fn swatch_center(
        rect: Rect,
        index: usize,
        columns: usize,
        dot_diameter: f32,
        dot_spacing: f32,
    ) -> Pos2 {
        Self::swatch_center_grid(rect, index, columns, columns, 1, dot_diameter, dot_spacing, dot_spacing)
    }

    /// Computes the exact center position of swatch `index` inside a multi-row grid within `rect`.
    #[allow(clippy::too_many_arguments)]
    pub fn swatch_center_grid(
        rect: Rect,
        index: usize,
        _total_count: usize,
        columns: usize,
        _rows: usize,
        dot_diameter: f32,
        col_gap: f32,
        row_gap: f32,
    ) -> Pos2 {
        let cols = columns.max(1);
        let (col, row) = (index % cols, index / cols);
        let (ring_pad_x, ring_pad_y) = (col_gap * 0.5, row_gap * 0.5);
        let radius = dot_diameter * 0.5;
        Pos2::new(
            rect.min.x + ring_pad_x + radius + (col as f32) * (dot_diameter + col_gap),
            rect.min.y + ring_pad_y + radius + (row as f32) * (dot_diameter + row_gap),
        )
    }

    /// Renders the tag color picker into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl Widget for MacTagColorPicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let tm = theme.tag_color_picker();
        let (def_diam, def_spacing) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (tm.dot_diameter.points(), tm.dot_spacing.points()),
            MacControlSize::Small | MacControlSize::CompactInfobox => (tm.small_dot_diameter.points(), tm.small_dot_spacing.points()),
            MacControlSize::Mini => (tm.mini_dot_diameter.points(), tm.mini_dot_spacing.points()),
        };
        let diam = self.custom_dot_size.unwrap_or(def_diam);
        let (col_gap, row_gap) = self.custom_spacing.unwrap_or((def_spacing, def_spacing));

        let default_palette = theme.tag_label_palette();
        let colors: &[Color32] = self.palette.unwrap_or(&default_palette);
        let color_count = colors.len();
        let total_count = color_count + usize::from(self.include_clear_swatch);
        let cols = if let Some(r) = self.rows {
            total_count.div_ceil(r.max(1)).max(1)
        } else {
            self.columns.max(1).min(total_count.max(1))
        };
        let rows = total_count.div_ceil(cols);
        let (pitch_x, pitch_y) = (diam + col_gap, diam + row_gap);
        let max_pitch = pitch_x.max(pitch_y);
        let desired_size = Vec2::new((cols as f32) * pitch_x, (rows as f32) * pitch_y);

        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_tag_color_picker_fixed");
            ui.skip_ahead_auto_ids(1);
            (snapped, ui.interact(snapped, id, sense))
        } else {
            let (r, resp) = ui.allocate_exact_size(desired_size, sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        if enabled && response.clicked() && total_count > 0 {
            let click_pos = response.interact_pointer_pos().unwrap_or(rect.center());
            let (mut best_idx, mut best_dist) = (0usize, f32::INFINITY);
            for idx in 0..total_count {
                let d = Self::swatch_center_grid(rect, idx, total_count, cols, rows, diam, col_gap, row_gap).distance(click_pos);
                if d < best_dist {
                    best_dist = d;
                    best_idx = idx;
                }
            }
            if best_dist <= max_pitch {
                let next_sel = (!self.include_clear_swatch || best_idx != color_count).then_some(best_idx);
                if *self.selected != next_sel {
                    *self.selected = next_sel;
                    response.mark_changed();
                }
            }
            response.request_focus();
        }

        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let surface = theme.active_surface_at(ui, rect.center());
        let hover_pos = ui.input(|i| i.pointer.hover_pos());
        let widget_hovered = enabled && (response.hovered() || hover_pos.is_some_and(|hp| rect.contains(hp)));
        let radius = diam * 0.5;
        let ring_r = radius + tm.selected_ring_offset.points();
        let ring_w = tm.selected_ring_stroke.points();
        let border_w = tm.border_stroke_width.points();
        let swatch_border = theme.tag_swatch_border().gamma_multiply(alpha);

        for idx in 0..total_count {
            let center = Self::swatch_center_grid(rect, idx, total_count, cols, rows, diam, col_gap, row_gap);
            let hovered = widget_hovered && hover_pos.is_some_and(|hp| hp.distance(center) <= max_pitch * 0.5);
            let is_selected_color = *self.selected == Some(idx);
            let sw_r = if hovered && !is_selected_color { radius + tm.hover_radius_delta.points() } else { radius };
            if self.include_clear_swatch && idx == color_count {
                let clear_fg = if hovered || self.selected.is_none() {
                    theme.label_on(surface)
                } else {
                    theme.secondary_label_on(surface)
                };
                let clear_stroke_c = clear_fg.gamma_multiply(alpha);
                painter.circle(center, sw_r, theme.tag_labels().clear.to_color32(), Stroke::new(border_w, clear_stroke_c));
                let cross_inset = sw_r * (1.0 - tm.clear_cross_inset_ratio.points());
                let cross_stroke = Stroke::new(tm.clear_cross_stroke_width.points(), clear_stroke_c);
                painter.line_segment([Pos2::new(center.x - cross_inset, center.y - cross_inset), Pos2::new(center.x + cross_inset, center.y + cross_inset)], cross_stroke);
                painter.line_segment([Pos2::new(center.x - cross_inset, center.y + cross_inset), Pos2::new(center.x + cross_inset, center.y - cross_inset)], cross_stroke);
            } else if let Some(&swatch_c) = colors.get(idx) {
                let fill = swatch_c.gamma_multiply(alpha);
                painter.circle(center, sw_r, fill, Stroke::new(border_w, swatch_border));
                if is_selected_color {
                    painter.circle_stroke(center, ring_r, Stroke::new(ring_w, fill));
                    let (min_x, min_y) = (center.x - radius, center.y - radius);
                    let p1 = Pos2::new(min_x + diam * tm.check_p1_x.points(), min_y + diam * tm.check_p1_y.points());
                    let p2 = Pos2::new(min_x + diam * tm.check_p2_x.points(), min_y + diam * tm.check_p2_y.points());
                    let p3 = Pos2::new(min_x + diam * tm.check_p3_x.points(), min_y + diam * tm.check_p3_y.points());
                    let on_acc = theme.on_accent_text();
                    let chk_col = if crate::theme::contrast_ratio(on_acc, swatch_c) < 2.0 {
                        crate::theme::MacBackgroundStyle::Light.primary_text_color(theme)
                    } else {
                        on_acc
                    };
                    let chk_stroke = Stroke::new(tm.checkmark_stroke_width.points(), chk_col.gamma_multiply(alpha));
                    painter.line_segment([p1, p2], chk_stroke);
                    painter.line_segment([p2, p3], chk_stroke);
                }
            }
        }
        response
    }
}
