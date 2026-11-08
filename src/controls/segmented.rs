//! Authentic macOS NSSegmentedControl primitive.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded metrics.

use crate::theme::MacTheme;
use egui::{
    CornerRadius, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

/// Authentic macOS segmented pill selector (`NSSegmentedControl` equivalent).
///
/// Features animated sliding selection indicator pill via `CASpring`, token-driven padding and typography,
/// ambient drop shadow, keyboard arrow navigation, and crisp border separators.
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacSegmentedControl, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut selected = 1;
/// let tabs = ["Day", "Week", "Month", "Year"];
///
/// MacSegmentedControl::new(&tabs, selected)
///     .theme(&theme)
///     .show(ui, |new_idx| {
///         selected = new_idx;
///     });
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacSegmentedControl<'a> {
    options: &'a [&'a str],
    selected_idx: usize,
    enabled: bool,
    theme: Option<&'a MacTheme>,
    height: Option<f32>,
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
        }
    }

    /// Enables or disables the segmented control.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides control height in points (defaults to token segmented height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Renders the segmented control and fires `on_select` when a new segment is clicked.
    pub fn show(self, ui: &mut Ui, mut on_select: impl FnMut(usize)) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::from_ctx(ui.ctx());
                &fallback_theme
            }
        };

        let segmented_tokens = theme.segmented();
        let metrics = &segmented_tokens.regular;
        let height = self.height.unwrap_or_else(|| metrics.height.points());
        let corner_radius = metrics.corner_radius.points();
        let indicator_radius = metrics.indicator_radius.points();
        let border_width = segmented_tokens.border_width.points();
        let indicator_inset = segmented_tokens.indicator_inset.points();

        let font_id = egui::FontId::proportional(segmented_tokens.font_size.points());
        let padding_h = segmented_tokens.padding_horizontal.points();
        let min_item_w = segmented_tokens.item_width.points();

        let mut max_content_w = min_item_w;
        for opt in self.options.iter() {
            let galley =
                ui.painter()
                    .layout_no_wrap((*opt).to_string(), font_id.clone(), theme.label());
            let padded_w = galley.size().x + padding_h * 2.0;
            if padded_w > max_content_w {
                max_content_w = padded_w;
            }
        }
        let segment_width = max_content_w;
        let count = self.options.len().max(1);
        let total_width = segment_width * count as f32;

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(total_width, height), sense);
        let mut active_idx = self.selected_idx.min(count.saturating_sub(1));

        if self.enabled && response.has_focus() {
            if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight))
                && active_idx + 1 < count
            {
                active_idx += 1;
                on_select(active_idx);
                response.mark_changed();
            } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft))
                && active_idx > 0
            {
                active_idx -= 1;
                on_select(active_idx);
                response.mark_changed();
            }
        }

        let alpha_factor = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };
        let painter = ui.painter_at(rect);

        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            theme.control_background().gamma_multiply(alpha_factor),
            Stroke::new(border_width, theme.separator().gamma_multiply(alpha_factor)),
            StrokeKind::Inside,
        );

        // CASpring sliding pill animation stored in egui temp memory
        let spring = theme.motion().swiftui_snappy.to_spring();
        let now = ui.input(|i| i.time);
        let spring_key = response.id.with("seg_caspring");
        let target_idx = active_idx as f32;
        let (prev_target, start_val, start_time) = ui.data(|d| {
            d.get_temp::<(f32, f32, f64)>(spring_key)
                .unwrap_or((target_idx, target_idx, now))
        });

        let anim_idx = if (prev_target - target_idx).abs() > f32::EPSILON {
            let elapsed = (now - start_time).max(0.0) as f32;
            let cur = start_val + (prev_target - start_val) * spring.position_at(elapsed);
            ui.data_mut(|d| d.insert_temp(spring_key, (target_idx, cur, now)));
            ui.ctx().request_repaint();
            cur
        } else {
            let elapsed = (now - start_time).max(0.0) as f32;
            if (target_idx - start_val).abs() > f32::EPSILON
                && elapsed < spring.settling_duration()
            {
                ui.ctx().request_repaint();
                start_val + (target_idx - start_val) * spring.position_at(elapsed)
            } else {
                target_idx
            }
        };

        let pill_x = rect.min.x + anim_idx * segment_width;
        let animated_seg_rect = Rect::from_min_size(
            Pos2::new(pill_x, rect.min.y),
            Vec2::new(segment_width, height),
        );
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

        for (i, opt) in self.options.iter().enumerate() {
            let seg_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * segment_width, rect.min.y),
                Vec2::new(segment_width, height),
            );

            if self.enabled && response.clicked() {
                if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                    if seg_rect.contains(pos) {
                        response.request_focus();
                        on_select(i);
                        response.mark_changed();
                    }
                }
            }

            let is_hovered = self.enabled && ui.rect_contains_pointer(seg_rect);
            let text_color = if i == active_idx || is_hovered {
                theme.label().gamma_multiply(alpha_factor)
            } else {
                theme.secondary_label().gamma_multiply(alpha_factor)
            };

            painter.text(
                seg_rect.center(),
                egui::Align2::CENTER_CENTER,
                *opt,
                font_id.clone(),
                text_color,
            );
        }

        response
    }
}
