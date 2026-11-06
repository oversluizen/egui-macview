//! Authentic macOS NSSegmentedControl primitive.
//!
//! Grounded in macOS HIG and W3C DTCG tokens via [`MacTheme`].
//! Fully data-driven: zero hardcoded metrics.

use crate::theme::MacTheme;
use egui::{Color32, CornerRadius, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2};

/// Authentic macOS segmented pill selector (`NSSegmentedControl` equivalent).
///
/// Features animated sliding selection indicator pill, token-driven padding and typography,
/// ambient drop shadow, and crisp border separators.
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
    theme: Option<&'a MacTheme>,
    height: Option<f32>,
}

impl<'a> MacSegmentedControl<'a> {
    /// Creates a segmented control with option labels and currently selected index.
    pub fn new(options: &'a [&'a str], selected_idx: usize) -> Self {
        Self {
            options,
            selected_idx,
            theme: None,
            height: None,
        }
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
                fallback_theme = MacTheme::default();
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
            let galley = ui.painter().layout_no_wrap(
                (*opt).to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
            );
            let label_w = galley.size().x;
            let padded_w = label_w + padding_h + padding_h;
            if padded_w > max_content_w {
                max_content_w = padded_w;
            }
        }
        let segment_width = max_content_w;
        let count = self.options.len().max(1);
        let total_width = segment_width * count as f32;

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(total_width, height), egui::Sense::click());

        let painter = ui.painter_at(rect);

        // Container background
        painter.rect(
            rect,
            CornerRadius::same(corner_radius as u8),
            theme.control_background(),
            Stroke::new(border_width, theme.separator()),
            StrokeKind::Inside,
        );

        // Smoothly animated segment index offset via egui animation
        let anim_idx = ui.ctx().animate_value_with_time(
            response.id.with("seg_anim"),
            self.selected_idx as f32,
            segmented_tokens.animation_duration.points(),
        );

        let pill_x = rect.min.x + anim_idx * segment_width;
        let animated_seg_rect = Rect::from_min_size(
            Pos2::new(pill_x, rect.min.y),
            Vec2::new(segment_width, height),
        );
        let pill_rect = animated_seg_rect.shrink(indicator_inset);

        // Ambient shadow and active segment pill
        painter.rect_filled(
            pill_rect.translate(Vec2::new(0.0, 1.0)),
            CornerRadius::same(indicator_radius as u8),
            Color32::from_black_alpha(20),
        );

        painter.rect(
            pill_rect,
            CornerRadius::same(indicator_radius as u8),
            theme.control(),
            Stroke::new(border_width, theme.separator()),
            StrokeKind::Inside,
        );

        for (i, opt) in self.options.iter().enumerate() {
            let seg_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * segment_width, rect.min.y),
                Vec2::new(segment_width, height),
            );

            // Click detection
            if response.clicked() {
                if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                    if seg_rect.contains(pos) {
                        on_select(i);
                    }
                }
            }

            let text_color = if i == self.selected_idx {
                theme.label()
            } else {
                theme.secondary_label()
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
