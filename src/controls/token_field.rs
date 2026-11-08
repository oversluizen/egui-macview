//! Authentic macOS AppKit Token Field (`NSTokenField` equivalent, SPEC-011).
//!
//! Grounded in Apple AppKit `NSTokenField` / `NSTokenFieldCell` and W3C DTCG tokens via [`MacTheme`].
//! Features `.rounded` 16pt token capsules, flow-wrapping layout, delimiter-driven commit (`,` or `Enter`),
//! `Backspace` and `×` button removal, outer focus ring halo, and optional character budget counter.

use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    Align2, CornerRadius, FontId, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke,
    StrokeKind, TextEdit, Ui, UiBuilder, Vec2, Widget,
};
use std::fmt::Debug;
use std::hash::Hash;

/// Authentic macOS AppKit token pill field (`NSTokenField` equivalent).
///
/// # Example
///
/// ```rust
/// use egui_macview::{MacTheme, MacTokenField};
///
/// # let ctx = egui::Context::default();
/// # let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::default();
/// let mut tokens = vec!["AppKit".to_string(), "AVKit".to_string()];
/// let mut input = String::new();
///
/// MacTokenField::new("keywords", &mut tokens, &mut input)
///     .theme(&theme)
///     .max_chars(120)
///     .show(ui);
/// # });
/// # output.textures_delta.clear();
/// ```
pub struct MacTokenField<'a> {
    id: Id,
    tokens: &'a mut Vec<String>,
    input_buffer: &'a mut String,
    placeholder: &'a str,
    max_chars: Option<usize>,
    width: Option<f32>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacTokenField<'a> {
    /// Creates a new `MacTokenField` bound to `tokens` and `input_buffer`.
    pub fn new(
        id_source: impl Hash + Debug,
        tokens: &'a mut Vec<String>,
        input_buffer: &'a mut String,
    ) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_source);
        Self {
            id,
            tokens,
            input_buffer,
            placeholder: "Add token...",
            max_chars: None,
            width: None,
            enabled: true,
            theme: None,
        }
    }

    /// Sets placeholder text shown when no tokens or input text are present.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Configures an optional character budget counter (`current / max_chars`) at bottom-right.
    pub fn max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = Some(max_chars);
        self
    }

    /// Overrides the control width in points.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Enables or disables the token field.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the token field in `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacTokenField<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let fallback_theme;
        let theme = match self.theme {
            Some(t) => t,
            None => {
                fallback_theme = MacTheme::default();
                &fallback_theme
            }
        };

        let tfm = theme.token_field();
        let pad_h = tfm.padding_horizontal.points();
        let pad_v = tfm.padding_vertical.points();
        let pill_h = tfm.pill_height.points();
        let pill_r = CornerRadius::same(tfm.pill_corner_radius.points() as u8);
        let pill_pad_h = tfm.pill_padding_horizontal.points();
        let pill_spacing = tfm.pill_spacing.points();
        let row_spacing = tfm.row_spacing.points();
        let font_sz = tfm.font_size.points();
        let remove_sz = tfm.remove_icon_size.points();
        let min_input_w = tfm.min_input_width.points();
        let width = self.width.unwrap_or_else(|| tfm.default_width.points());

        let mut changed = false;

        // 1. Pre-process comma or newline delimiters in input_buffer
        if self.enabled && (self.input_buffer.contains(',') || self.input_buffer.contains('\n')) {
            let raw = std::mem::take(self.input_buffer);
            let segments: Vec<&str> = raw.split([',', '\n', '\r']).collect();
            for (idx, seg) in segments.iter().enumerate() {
                let trimmed = seg.trim();
                if idx + 1 < segments.len() {
                    if !trimmed.is_empty() {
                        self.tokens.push(trimmed.to_string());
                        changed = true;
                    }
                } else {
                    *self.input_buffer = trimmed.to_string();
                }
            }
        }

        let input_id = self.id.with("token_input");

        // 2. Handle Enter and Backspace keys before TextEdit processes them
        if self.enabled {
            if !self.input_buffer.trim().is_empty()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter))
            {
                let committed = self.input_buffer.trim().to_string();
                self.tokens.push(committed);
                self.input_buffer.clear();
                changed = true;
            } else if self.input_buffer.is_empty()
                && !self.tokens.is_empty()
                && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Backspace))
            {
                self.tokens.pop();
                changed = true;
            }
        }

        // 3. Measure pill widths and compute flow-wrapping layout
        let font_id = FontId::proportional(font_sz);
        let max_inner_w = (width - pad_h * 2.0).max(min_input_w);
        let mut pill_layouts: Vec<(usize, f32, f32, f32)> = Vec::with_capacity(self.tokens.len());
        let mut cursor_x = 0.0f32;
        let mut cursor_y = 0.0f32;

        for (idx, tok) in self.tokens.iter().enumerate() {
            let text_w = ui.fonts_mut(|f| {
                f.layout_no_wrap(tok.clone(), font_id.clone(), theme.label())
                    .size()
                    .x
            });
            let w = (pill_pad_h * 2.0 + text_w + remove_sz).min(max_inner_w);
            if cursor_x > 0.0 && cursor_x + w > max_inner_w {
                cursor_x = 0.0;
                cursor_y += pill_h + row_spacing;
            }
            pill_layouts.push((idx, cursor_x, cursor_y, w));
            cursor_x += w + pill_spacing;
        }

        if cursor_x > 0.0 && cursor_x + min_input_w > max_inner_w {
            cursor_x = 0.0;
            cursor_y += pill_h + row_spacing;
        }
        let input_rel_x = cursor_x;
        let input_rel_y = cursor_y;
        let input_w = (max_inner_w - input_rel_x).max(min_input_w);

        let counter_extra_h = if self.max_chars.is_some() {
            tfm.counter_row_height.points()
        } else {
            0.0
        };
        let content_h = input_rel_y + pill_h + pad_v * 2.0 + counter_extra_h;
        let total_h = content_h.max(tfm.min_height.points());

        let (raw_rect, response) =
            ui.allocate_exact_size(Vec2::new(width, total_h), Sense::click());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());

        if response.clicked() && self.enabled {
            ui.memory_mut(|m| m.request_focus(input_id));
        }

        let input_rect = Rect::from_min_size(
            Pos2::new(rect.left() + pad_h + input_rel_x, rect.top() + pad_v + input_rel_y),
            Vec2::new(input_w, pill_h),
        );

        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(input_rect));
        let edit_resp = child_ui.add_sized(
            input_rect.size(),
            TextEdit::singleline(self.input_buffer)
                .id(input_id)
                .font(font_id.clone())
                .text_color(if self.enabled {
                    theme.label()
                } else {
                    theme.placeholder_text()
                })
                .frame(egui::Frame::NONE)
                .interactive(self.enabled),
        );

        if edit_resp.changed() && self.input_buffer.contains(',') {
            let raw = std::mem::take(self.input_buffer);
            for part in raw.split(',') {
                let t = part.trim();
                if !t.is_empty() {
                    self.tokens.push(t.to_string());
                    changed = true;
                }
            }
        }

        let is_focused =
            edit_resp.has_focus() || response.has_focus() || ui.memory(|m| m.has_focus(input_id));
        let painter = ui.painter_at(rect);
        let bezel_r = tfm.corner_radius.points();

        if self.enabled && is_focused {
            let ring = theme.focus_ring();
            let focus_rect = rect.expand(ring.offset.points());
            let focus_r = bezel_r + ring.corner_radius_delta.points();
            painter.rect_stroke(
                focus_rect,
                CornerRadius::same(focus_r as u8),
                Stroke::new(ring.stroke_width.points(), theme.keyboard_focus_indicator()),
                StrokeKind::Outside,
            );
        }

        let border_color = if is_focused {
            theme.control_accent()
        } else {
            theme.separator()
        };
        painter.rect(
            rect,
            CornerRadius::same(bezel_r as u8),
            theme.control_background(),
            Stroke::new(tfm.border_width.points(), border_color),
            StrokeKind::Inside,
        );

        let mut remove_idx = None;
        for &(idx, rel_x, rel_y, w) in &pill_layouts {
            if idx >= self.tokens.len() {
                continue;
            }
            let pill_rect = Rect::from_min_size(
                Pos2::new(rect.left() + pad_h + rel_x, rect.top() + pad_v + rel_y),
                Vec2::new(w, pill_h),
            );
            let rm_rect = Rect::from_center_size(
                Pos2::new(
                    pill_rect.right() - pill_pad_h * 0.5 - remove_sz * 0.5,
                    pill_rect.center().y,
                ),
                Vec2::splat(remove_sz),
            );
            let rm_resp = ui.interact(
                rm_rect,
                self.id.with(("rm_tok", idx)),
                if self.enabled {
                    Sense::click()
                } else {
                    Sense::hover()
                },
            );
            if rm_resp.clicked() && self.enabled {
                remove_idx = Some(idx);
            }

            let pill_bg = if rm_resp.hovered() && self.enabled {
                theme.selected_control()
            } else {
                theme.control()
            };
            painter.rect(
                pill_rect,
                pill_r,
                pill_bg,
                Stroke::new(tfm.border_width.points(), theme.separator()),
                StrokeKind::Inside,
            );

            painter.text(
                Pos2::new(pill_rect.left() + pill_pad_h, pill_rect.center().y),
                Align2::LEFT_CENTER,
                &self.tokens[idx],
                font_id.clone(),
                theme.label(),
            );

            let c = rm_rect.center();
            let span = tfm.remove_cross_span.points();
            let x_stroke = Stroke::new(tfm.remove_icon_stroke.points(), theme.secondary_label());
            painter.line_segment(
                [Pos2::new(c.x - span, c.y - span), Pos2::new(c.x + span, c.y + span)],
                x_stroke,
            );
            painter.line_segment(
                [Pos2::new(c.x - span, c.y + span), Pos2::new(c.x + span, c.y - span)],
                x_stroke,
            );
        }

        if let Some(idx) = remove_idx {
            if idx < self.tokens.len() {
                self.tokens.remove(idx);
                changed = true;
            }
        }

        if self.tokens.is_empty() && self.input_buffer.is_empty() && !is_focused {
            painter.text(
                Pos2::new(input_rect.left(), input_rect.center().y),
                Align2::LEFT_CENTER,
                self.placeholder,
                font_id,
                theme.placeholder_text(),
            );
        }

        if let Some(max_c) = self.max_chars {
            let total_c: usize = self.tokens.iter().map(|t| t.chars().count()).sum::<usize>()
                + self.input_buffer.chars().count();
            let counter_color = if total_c > max_c {
                theme.system_red()
            } else {
                theme.secondary_label()
            };
            painter.text(
                Pos2::new(rect.right() - pad_h, rect.bottom() - pad_v * 0.5),
                Align2::RIGHT_BOTTOM,
                format!("{total_c} / {max_c}"),
                FontId::proportional(tfm.counter_font_size.points()),
                counter_color,
            );
        }

        let mut final_resp = edit_resp.union(response);
        if changed {
            final_resp.mark_changed();
        }
        final_resp
    }
}
