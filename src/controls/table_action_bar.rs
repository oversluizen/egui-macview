//! Authentic macOS AppKit `NSBezelStyleSmallSquare` `[ + | - | action ]` Table Action Bar (`MacTableActionBar`).

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{CornerRadius, FontId, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::hash::Hash;

/// Interaction result returned by [`MacTableActionBar::show`].
#[derive(Debug, Clone)]
pub struct MacTableActionResponse {
    /// Combined bar [`egui::Response`].
    pub response: Response,
    /// `true` when the leading `[ + ]` segment was clicked while enabled and `can_add` is `true`.
    pub add_clicked: bool,
    /// `true` when the `[ - ]` segment was clicked while enabled and `can_remove` is `true`.
    pub remove_clicked: bool,
    /// `true` when the optional `[ action ]` gear/menu segment was clicked while enabled.
    pub action_clicked: bool,
}

/// Authentic macOS AppKit `NSBezelStyleSmallSquare` `[ + | - ]` Action Strip at the bottom of lists and tables.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacTableActionBar, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let resp = MacTableActionBar::new("layers_bar").theme(&theme).show(ui);
/// if resp.add_clicked {}
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacTableActionBar<'a> {
    id: Id,
    can_add: bool,
    can_remove: bool,
    with_action_menu: bool,
    fill_remaining_strip: bool,
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTableActionBar<'a> {
    /// Creates a new `NSBezelStyleSmallSquare` `[ + | - ]` action bar with the given `id_source`.
    pub fn new(id_source: impl Hash) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id_source.hash(&mut hasher);
        Self {
            id: Id::NULL.with(std::hash::Hasher::finish(&hasher)),
            can_add: true,
            can_remove: true,
            with_action_menu: false,
            fill_remaining_strip: true,
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Enables or disables the `[ + ]` segment (default `true`).
    pub fn can_add(mut self, can_add: bool) -> Self {
        self.can_add = can_add;
        self
    }

    /// Enables or disables the `[ - ]` segment (default `true`).
    pub fn can_remove(mut self, can_remove: bool) -> Self {
        self.can_remove = can_remove;
        self
    }

    /// Appends a 3rd `[ action ]` gear/menu segment when `true` (default `false`).
    pub fn with_action_menu(mut self, with_action_menu: bool) -> Self {
        self.with_action_menu = with_action_menu;
        self
    }

    /// When `true` (default), spans the full available width as a continuous AppKit footer strip;
    /// when `false`, allocates only the compact `[ + | - ]` button cluster width.
    pub fn fill_remaining_strip(mut self, fill_remaining: bool) -> Self {
        self.fill_remaining_strip = fill_remaining;
        self
    }

    /// Sets the AppKit control size (`Regular` `22pt`, `Small` `18pt`, `Mini` `15pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the table action bar to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Enables or disables the entire action bar.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Computes the exact center position of `+` / `-` button segment `index` (`0 = +`, `1 = -`) inside `rect`.
    pub fn button_center(rect: Rect, index: usize, button_width: f32) -> Pos2 {
        Pos2::new(rect.min.x + ((index as f32) + 0.5) * button_width, rect.center().y)
    }

    /// Computes the exact center position of the 3rd `[ action ]` gear/menu segment inside `rect`.
    pub fn action_button_center(rect: Rect, button_width: f32, action_button_width: f32) -> Pos2 {
        Pos2::new(
            rect.min.x + button_width * 2.0 + action_button_width * 0.5,
            rect.center().y,
        )
    }

    /// Renders the table action bar into `ui` and returns [`MacTableActionResponse`].
    pub fn show(self, ui: &mut Ui) -> MacTableActionResponse {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let ab = theme.table_action_bar();

        let (bar_h, btn_w, act_w, glyph_sz, arm_half) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (
                ab.bar_height.points(), ab.button_width.points(), ab.action_button_width.points(),
                ab.glyph_size.points(), ab.glyph_arm_half.points(),
            ),
            MacControlSize::Small | MacControlSize::CompactInfobox => (
                ab.small_bar_height.points(), ab.small_button_width.points(), ab.small_action_button_width.points(),
                ab.small_glyph_size.points(), ab.small_glyph_arm_half.points(),
            ),
            MacControlSize::Mini => (
                ab.mini_bar_height.points(), ab.mini_button_width.points(), ab.mini_action_button_width.points(),
                ab.small_glyph_size.points(), ab.small_glyph_arm_half.points(),
            ),
        };

        let cluster_w = if self.with_action_menu { btn_w * 2.0 + act_w } else { btn_w * 2.0 };
        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();

        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            (snapped, ui.interact(snapped, self.id.with("bar"), sense))
        } else {
            let width = if self.fill_remaining_strip { ui.available_width().max(cluster_w) } else { cluster_w };
            let (r, resp) = ui.allocate_exact_size(Vec2::new(width, bar_h), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        let visual_rect = theme.clamp_nib_visual_rect(rect, bar_h, ppp);
        let add_rect = Rect::from_min_size(visual_rect.min, Vec2::new(btn_w, visual_rect.height())).intersect(visual_rect);
        let rem_rect = Rect::from_min_size(Pos2::new(visual_rect.min.x + btn_w, visual_rect.min.y), Vec2::new(btn_w, visual_rect.height())).intersect(visual_rect);
        let act_rect = Rect::from_min_size(Pos2::new(visual_rect.min.x + btn_w * 2.0, visual_rect.min.y), Vec2::new(act_w, visual_rect.height())).intersect(visual_rect);

        let (mut add_clicked, mut remove_clicked, mut action_clicked) = (false, false, false);
        if enabled && response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if self.can_add && add_rect.is_positive() && add_rect.contains(pos) {
                    add_clicked = true;
                } else if self.can_remove && rem_rect.is_positive() && rem_rect.contains(pos) {
                    remove_clicked = true;
                } else if self.with_action_menu && act_rect.is_positive() && act_rect.contains(pos) {
                    action_clicked = true;
                }
            }
            if add_clicked || remove_clicked || action_clicked {
                response.mark_changed();
            }
        }

        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(visual_rect);
        let radius = CornerRadius::same(ab.corner_radius.points() as u8);
        let border_w = ab.border_width.points();
        let border_stroke = Stroke::new(border_w, theme.separator().gamma_multiply(alpha));

        painter.rect_filled(visual_rect, radius, theme.box_background().gamma_multiply(alpha));
        let active_cluster_rect = Rect::from_min_size(
            visual_rect.min,
            Vec2::new(cluster_w.min(visual_rect.width()), visual_rect.height()),
        );
        painter.rect_filled(active_cluster_rect, radius, theme.control().gamma_multiply(alpha));

        let hover_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_down = response.is_pointer_button_down_on();
        for (seg_r, seg_ok) in [
            (add_rect, enabled && self.can_add),
            (rem_rect, enabled && self.can_remove),
            (act_rect, enabled && self.with_action_menu),
        ] {
            if seg_ok && seg_r.is_positive() && hover_pos.is_some_and(|p| seg_r.contains(p)) {
                let hi = if pointer_down {
                    theme.selected_control_inactive()
                } else {
                    theme.control_background()
                };
                painter.rect_filled(seg_r, CornerRadius::ZERO, hi);
            }
        }

        let div_xs = [
            visual_rect.min.x + btn_w,
            visual_rect.min.x + btn_w * 2.0,
            visual_rect.min.x + cluster_w,
        ];
        let div_count = if self.with_action_menu { 3 } else { 2 };
        for &x in &div_xs[..div_count] {
            if x < visual_rect.max.x - border_w {
                painter.line_segment(
                    [Pos2::new(x, visual_rect.top()), Pos2::new(x, visual_rect.bottom())],
                    border_stroke,
                );
            }
        }
        painter.rect_stroke(visual_rect, radius, border_stroke, StrokeKind::Inside);

        let surface = theme.control();
        let base_fg = theme.label_on(surface);
        let glyph_w = ab.glyph_stroke_width.points();

        if add_rect.width() >= btn_w * 0.5 {
            let add_alpha = if enabled && self.can_add { 1.0 } else { theme.disabled_opacity() };
            let add_stroke = Stroke::new(glyph_w, base_fg.gamma_multiply(add_alpha));
            let ac = add_rect.center();
            painter.line_segment([Pos2::new(ac.x - arm_half, ac.y), Pos2::new(ac.x + arm_half, ac.y)], add_stroke);
            painter.line_segment([Pos2::new(ac.x, ac.y - arm_half), Pos2::new(ac.x, ac.y + arm_half)], add_stroke);
        }

        if rem_rect.width() >= btn_w * 0.5 {
            let rem_alpha = if enabled && self.can_remove { 1.0 } else { theme.disabled_opacity() };
            let rem_stroke = Stroke::new(glyph_w, base_fg.gamma_multiply(rem_alpha));
            let rc = rem_rect.center();
            painter.line_segment([Pos2::new(rc.x - arm_half, rc.y), Pos2::new(rc.x + arm_half, rc.y)], rem_stroke);
        }

        if self.with_action_menu && act_rect.width() >= act_w * 0.5 {
            let act_fg = base_fg.gamma_multiply(alpha);
            let galley = theme.layout_tracked_galley(
                ui,
                MacSymbol::Gear.as_str(),
                FontId::proportional(glyph_sz),
                act_fg,
                f32::INFINITY,
            );
            let gap = ab.action_chevron_gap.points();
            let chw = ab.action_chevron_half_width.points();
            let ch = ab.action_chevron_height.points();
            let total_w = galley.size().x + gap + chw * 2.0;
            let start_x = act_rect.center().x - total_w * 0.5;
            let gear_pos = Pos2::new(start_x, act_rect.center().y - galley.size().y * 0.5);
            painter.galley(gear_pos, galley, act_fg);
            let (cx, cy) = (start_x + total_w - chw, act_rect.center().y);
            let chev_stroke = Stroke::new(glyph_w, act_fg);
            painter.line_segment([Pos2::new(cx - chw, cy - ch * 0.5), Pos2::new(cx, cy + ch * 0.5)], chev_stroke);
            painter.line_segment([Pos2::new(cx, cy + ch * 0.5), Pos2::new(cx + chw, cy - ch * 0.5)], chev_stroke);
        }

        MacTableActionResponse { response, add_clicked, remove_clicked, action_clicked }
    }
}
