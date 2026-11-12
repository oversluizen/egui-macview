//! Authentic macOS NSSegmentedControl primitive.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded metrics.

use crate::controls::MacControlSize;
use crate::theme::MacTheme;
use egui::{
    Color32, CornerRadius, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2,
};

/// Authentic macOS segmented pill selector (`NSSegmentedControl` equivalent).
///
/// Features animated sliding selection indicator pill via `CASpring`, token-driven padding and typography,
/// ambient drop shadow, keyboard arrow navigation, and crisp border separators.
///
/// # Example
/// ```rust
/// use egui_macview::{MacSegmentedControl, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selected = 1;
/// MacSegmentedControl::new(&["Day", "Week", "Month", "Year"], selected).theme(&theme).show(ui, |i| selected = i);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSegmentedControl<'a> {
    options: &'a [&'a str],
    selected_idx: usize,
    enabled: bool,
    theme: Option<&'a MacTheme>,
    height: Option<f32>,
    control_size: MacControlSize,
    small_square: bool,
    fixed_rect: Option<Rect>,
}

impl<'a> MacSegmentedControl<'a> {
    /// Creates a segmented control with option labels and currently selected index.
    pub fn new(options: &'a [&'a str], selected_idx: usize) -> Self {
        Self {
            options,
            selected_idx,
            enabled: true,
            theme: None,
            height: None,
            control_size: MacControlSize::Regular,
            small_square: false,
            fixed_rect: None,
        }
    }

    /// Enables or disables the segmented control.
    pub fn enabled(mut self, enabled: bool) -> Self { self.enabled = enabled; self }
    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self { self.theme = Some(theme); self }
    /// Overrides control height in points (defaults to token segmented height).
    pub fn height(mut self, height: f32) -> Self { self.height = Some(height); self }
    /// Sets the AppKit control size (`Regular` `22pt`, `Small` `18pt`, or `Mini` `15pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self { self.control_size = size; self }
    /// Configures AppKit `NSSegmentedStyleSmallSquare` (`2.0pt` corner radius and internal vertical dividers).
    pub fn small_square(mut self) -> Self { self.small_square = true; self }
    /// Pins the segmented control to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self { self.fixed_rect = Some(rect); self }

    /// Renders a multi-select segmented strip (`NSSegmentedSwitchTrackingSelectAny`), toggling
    /// `active[i]` when segment `i` is clicked and returning `Some(i)` on toggle.
    pub fn show_multi(self, ui: &mut Ui, active: &mut [bool]) -> Option<usize> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let seg = theme.segmented();
        let (tier, font_sz) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (&seg.regular, seg.font_size.points()),
            MacControlSize::Small | MacControlSize::CompactInfobox => (&seg.small, seg.small_font_size.points()),
            MacControlSize::Mini => (&seg.mini, seg.mini_font_size.points()),
        };
        let height = self.fixed_rect.map(|r| r.height()).or(self.height).unwrap_or_else(|| tier.height.points());
        let corner_radius = if self.small_square {
            seg.small_square_corner_radius.points()
        } else {
            tier.corner_radius.points()
        };
        let border_w = seg.border_width.points();
        let font_id = egui::FontId::proportional(font_sz);
        let padding_h = seg.padding_horizontal.points();
        let mut max_content_w = seg.item_width.points();
        let mut galleys = Vec::with_capacity(self.options.len());
        for &opt in self.options {
            let g = theme.layout_tracked_galley(ui, opt, font_id.clone(), Color32::PLACEHOLDER, f32::INFINITY);
            max_content_w = max_content_w.max(g.size().x + padding_h * 2.0);
            galleys.push(g);
        }
        let count = self.options.len().max(1);
        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let id = ui.next_auto_id().with("mac_segmented_multi_fixed");
            ui.skip_ahead_auto_ids(1);
            (fixed, ui.interact(fixed, id, sense))
        } else {
            ui.allocate_exact_size(Vec2::new(max_content_w * count as f32, height), sense)
        };
        let segment_width = rect.width() / count as f32;
        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let border_stroke = Stroke::new(border_w, theme.box_border().gamma_multiply(alpha));
        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            theme.segmented_small_square_inactive_bg().gamma_multiply(alpha),
            border_stroke,
            StrokeKind::Inside,
        );

        let mut toggled = None;
        for (i, galley) in galleys.into_iter().enumerate() {
            let seg_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * segment_width, rect.min.y),
                Vec2::new(segment_width, rect.height()),
            );
            if enabled && response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if seg_rect.contains(pos) {
                        if let Some(slot) = active.get_mut(i) {
                            *slot = !*slot;
                        }
                        toggled = Some(i);
                        response.mark_changed();
                    }
                }
            }
            let is_on = active.get(i).copied().unwrap_or(false);
            if is_on {
                painter.rect_filled(
                    seg_rect.shrink(border_w),
                    CornerRadius::ZERO,
                    theme.segmented_small_square_active_bg().gamma_multiply(alpha),
                );
            }
            let text_color = if is_on {
                theme.label().gamma_multiply(alpha)
            } else {
                theme.secondary_label().gamma_multiply(alpha)
            };
            let text_pos = egui::Align2::CENTER_CENTER.anchor_size(seg_rect.center(), galley.size()).min;
            painter.galley(text_pos, galley, text_color);
        }
        for i in 1..count {
            let x = rect.min.x + i as f32 * segment_width;
            painter.line_segment([Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)], border_stroke);
        }
        toggled
    }

    /// Renders the segmented control and fires `on_select` when a new segment is clicked.
    pub fn show(self, ui: &mut Ui, mut on_select: impl FnMut(usize)) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let seg = theme.segmented();
        let (tier, font_sz) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (&seg.regular, seg.font_size.points()),
            MacControlSize::Small | MacControlSize::CompactInfobox => (&seg.small, seg.small_font_size.points()),
            MacControlSize::Mini => (&seg.mini, seg.mini_font_size.points()),
        };
        let height = self.fixed_rect.map(|r| r.height()).or(self.height).unwrap_or_else(|| tier.height.points());
        let corner_radius = if self.small_square {
            seg.small_square_corner_radius.points()
        } else {
            tier.corner_radius.points()
        };
        let indicator_radius = tier.indicator_radius.points();
        let border_width = seg.border_width.points();
        let indicator_inset = seg.indicator_inset.points();
        let font_id = egui::FontId::proportional(font_sz);
        let padding_h = seg.padding_horizontal.points();
        let mut max_content_w = seg.item_width.points();
        let mut galleys = Vec::with_capacity(self.options.len());
        for &opt in self.options {
            let galley = theme.layout_tracked_galley(ui, opt, font_id.clone(), Color32::PLACEHOLDER, f32::INFINITY);
            max_content_w = max_content_w.max(galley.size().x + padding_h * 2.0);
            galleys.push(galley);
        }
        let count = self.options.len().max(1);
        let enabled = self.enabled && ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let id = ui.next_auto_id().with("mac_segmented_fixed");
            ui.skip_ahead_auto_ids(1);
            (fixed, ui.interact(fixed, id, sense))
        } else {
            ui.allocate_exact_size(Vec2::new(max_content_w * count as f32, height), sense)
        };
        let segment_width = rect.width() / count as f32;
        let mut active_idx = self.selected_idx.min(count.saturating_sub(1));

        if enabled && response.has_focus() {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) && active_idx + 1 < count {
                active_idx += 1;
                on_select(active_idx);
                response.mark_changed();
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) && active_idx > 0 {
                active_idx -= 1;
                on_select(active_idx);
                response.mark_changed();
            }
        }

        let alpha_factor = if enabled { 1.0 } else { theme.disabled_opacity() };
        let painter = ui.painter_at(rect);
        let bg_fill = if self.small_square {
            theme.segmented_small_square_inactive_bg()
        } else {
            theme.control_background()
        };
        let border_col = if self.small_square { theme.box_border() } else { theme.separator() };
        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            bg_fill.gamma_multiply(alpha_factor),
            Stroke::new(border_width, border_col.gamma_multiply(alpha_factor)),
            StrokeKind::Inside,
        );

        let spring = theme.motion().swiftui_snappy.to_spring();
        let anim_idx = spring.animate_f32(ui.ctx(), response.id.with("seg_caspring"), active_idx as f32);
        let pill_x = rect.min.x + anim_idx * segment_width;
        let animated_seg_rect = Rect::from_min_size(Pos2::new(pill_x, rect.min.y), Vec2::new(segment_width, rect.height()));
        if self.small_square {
            painter.rect_filled(
                animated_seg_rect.shrink(border_width),
                CornerRadius::ZERO,
                theme.segmented_small_square_active_bg().gamma_multiply(alpha_factor),
            );
            for i in 1..count {
                let x = rect.min.x + i as f32 * segment_width;
                painter.line_segment(
                    [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                    Stroke::new(border_width, border_col.gamma_multiply(alpha_factor)),
                );
            }
        } else {
            let pill_rect = animated_seg_rect.shrink(indicator_inset);
            painter.rect_filled(
                pill_rect.translate(Vec2::new(0.0, border_width)),
                CornerRadius::same(indicator_radius as u8),
                theme.drop_shadow().gamma_multiply(alpha_factor),
            );
            painter.rect(
                pill_rect,
                CornerRadius::same(indicator_radius as u8),
                theme.control().gamma_multiply(alpha_factor),
                Stroke::new(border_width, theme.separator().gamma_multiply(alpha_factor)),
                StrokeKind::Inside,
            );
        }

        for (i, galley) in galleys.into_iter().enumerate() {
            let seg_rect = Rect::from_min_size(Pos2::new(rect.min.x + i as f32 * segment_width, rect.min.y), Vec2::new(segment_width, rect.height()));
            if enabled && response.clicked() {
                if let Some(pos) = response.interact_pointer_pos().or_else(|| ui.input(|inp| inp.pointer.interact_pos())) {
                    if seg_rect.contains(pos) {
                        response.request_focus();
                        on_select(i);
                        response.mark_changed();
                    }
                }
            }
            let is_hovered = enabled && ui.rect_contains_pointer(seg_rect);
            let text_color = if i == active_idx || is_hovered {
                theme.label().gamma_multiply(alpha_factor)
            } else {
                theme.secondary_label().gamma_multiply(alpha_factor)
            };
            let text_pos = egui::Align2::CENTER_CENTER.anchor_size(seg_rect.center(), galley.size()).min;
            painter.galley(text_pos, galley, text_color);
        }
        response
    }
}
