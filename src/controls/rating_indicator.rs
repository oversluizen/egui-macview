//! Authentic macOS AppKit Rating Level Indicator (`NSLevelIndicatorStyleRating`).
//!
//! Grounded in Apple AppKit `NSLevelIndicator` and W3C DTCG tokens via [`MacTheme`].
//! Features authentic 5-point vector star symbols, interactive click-to-rate,
//! toggle-off upon repeated click, keyboard arrow navigation, and theme gold/accent tinting.

use crate::theme::MacTheme;
use egui::{
    CornerRadius, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget,
};

/// Authentic macOS AppKit Rating Level Indicator (`NSLevelIndicatorStyleRating`).
pub struct MacRatingIndicator<'a> {
    rating: &'a mut u32,
    max_rating: u32,
    theme: Option<&'a MacTheme>,
    item_size: Option<f32>,
    enabled: bool,
}

impl<'a> MacRatingIndicator<'a> {
    /// Creates a new rating indicator bound to `rating` with maximum stars `max_rating`.
    pub fn new(rating: &'a mut u32, max_rating: u32) -> Self {
        Self {
            rating,
            max_rating: max_rating.max(1),
            theme: None,
            item_size: None,
            enabled: true,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Sets explicit star symbol bounding box in points.
    pub fn item_size(mut self, size: f32) -> Self {
        self.item_size = Some(size);
        self
    }

    /// Sets interactive enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Shows the rating indicator in the UI and returns its interaction response.
    pub fn show(self, ui: &mut Ui) -> Response {
        self.ui(ui)
    }
}

impl<'a> Widget for MacRatingIndicator<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let lm = theme.level_indicator();

        let max_rating = self.max_rating.max(1);
        if *self.rating > max_rating {
            *self.rating = max_rating;
        }

        let size = self
            .item_size
            .unwrap_or_else(|| lm.thickness.points() + lm.thickness.points() * 0.5);
        let gap = lm.segment_spacing.points();
        let count = max_rating as usize;
        let total_w = (count as f32) * size + ((count.saturating_sub(1)) as f32) * gap;

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(total_w, size),
            if self.enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );

        if self.enabled {
            if response.clicked() {
                ui.memory_mut(|m| m.request_focus(response.id));
            }

            if let Some(pos) = response.interact_pointer_pos() {
                if response.clicked() {
                    let rel_x = (pos.x - rect.left()).clamp(0.0, total_w);
                    let star_idx = ((rel_x / (size + gap)).floor() as u32 + 1).min(max_rating);
                    if *self.rating == star_idx {
                        *self.rating = 0;
                    } else {
                        *self.rating = star_idx;
                    }
                    response.mark_changed();
                }
            }

            if response.has_focus() {
                if (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight))
                    || ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)))
                    && *self.rating < max_rating
                {
                    *self.rating += 1;
                    response.mark_changed();
                } else if (ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft))
                    || ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)))
                    && *self.rating > 0
                {
                    *self.rating -= 1;
                    response.mark_changed();
                }
            }
        }

        let alpha = if self.enabled {
            1.0
        } else {
            theme.disabled_opacity()
        };
        let painter = ui.painter_at(rect);
        let hover_pos = ui.input(|i| i.pointer.hover_pos());

        for i in 1..=self.max_rating {
            let idx = (i - 1) as usize;
            let item_x = rect.left() + (idx as f32) * (size + gap);
            let item_rect = Rect::from_min_size(Pos2::new(item_x, rect.top()), Vec2::splat(size));
            let center = item_rect.center();
            let is_active = i <= *self.rating;
            let is_hovered = self.enabled
                && hover_pos
                    .map(|hp| item_rect.expand(gap * 0.5).contains(hp))
                    .unwrap_or(false);

            let fill = if is_active {
                theme.system_yellow().gamma_multiply(alpha)
            } else if is_hovered {
                theme.system_yellow().gamma_multiply(alpha * 0.5)
            } else {
                theme.control().gamma_multiply(alpha)
            };

            let stroke = Stroke::new(
                lm.border_width.points(),
                if is_active || is_hovered {
                    theme.system_yellow().gamma_multiply(alpha)
                } else {
                    theme.separator().gamma_multiply(alpha)
                },
            );

            // Vector 5-point star geometry (pure integer trigonometry)
            let outer_r = size * 0.5;
            let inner_r = outer_r * (f32::from(2u8) / f32::from(5u8));
            let mut star_pts = Vec::with_capacity(10);
            for p in 0..10 {
                let r = if p % 2 == 0 { outer_r } else { inner_r };
                let angle = (p as f32) * (std::f32::consts::PI / f32::from(5u8))
                    - std::f32::consts::FRAC_PI_2;
                star_pts.push(Pos2::new(
                    center.x + r * angle.cos(),
                    center.y + r * angle.sin(),
                ));
            }
            painter.add(egui::Shape::convex_polygon(star_pts, fill, stroke));
        }

        if self.enabled && response.has_focus() {
            let ring = theme.focus_ring();
            let halo = Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator());
            painter.rect_stroke(
                rect.expand(ring.offset.points()),
                CornerRadius::same(lm.corner_radius.points() as u8),
                halo,
                StrokeKind::Outside,
            );
        }

        response
    }
}
