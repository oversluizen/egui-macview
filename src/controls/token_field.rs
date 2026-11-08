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
        id_src: impl Hash + Debug,
        tokens: &'a mut Vec<String>,
        input_buffer: &'a mut String,
    ) -> Self {
        #[allow(deprecated)]
        let id = Id::new(id_src);
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

    /// Queries the currently selected token pill index for `id_source` in `ui`.
    pub fn selected_token(ui: &Ui, id_source: impl Hash + Debug) -> Option<usize> {
        #[allow(deprecated)]
        let id = Id::new(id_source).with("selected_token");
        ui.data(|d| d.get_temp::<Option<usize>>(id)).flatten()
    }
}

impl<'a> Widget for MacTokenField<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
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
        let sel_id = self.id.with("selected_token");
        let raw_sel = ui.data(|d| d.get_temp::<Option<usize>>(sel_id)).flatten();
        let mut selected_tok = raw_sel.filter(|&i| i < self.tokens.len());

        // 2. Handle Enter, Delete, Backspace, and Left/Right arrow keys
        if self.enabled {
            let has_text = !self.input_buffer.trim().is_empty();
            if has_text && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter)) {
                self.tokens.push(self.input_buffer.trim().to_string());
                self.input_buffer.clear();
                selected_tok = None;
                changed = true;
            } else if let Some(sel_idx) = selected_tok {
                let del = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete));
                let bs = ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Backspace));
                if del || bs {
                    self.tokens.remove(sel_idx);
                    selected_tok = None;
                    changed = true;
                } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
                    selected_tok = Some(sel_idx.saturating_sub(1));
                } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
                    selected_tok = (sel_idx + 1 < self.tokens.len()).then_some(sel_idx + 1);
                }
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
        let mut pill_layouts = Vec::with_capacity(self.tokens.len());
        let (mut cursor_x, mut cursor_y) = (0.0f32, 0.0f32);

        for (idx, tok) in self.tokens.iter().enumerate() {
            let f_clone = font_id.clone();
            let col = theme.label();
            let text_w = ui.fonts_mut(|f| f.layout_no_wrap(tok.clone(), f_clone, col).size().x);
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
        let (input_rel_x, input_rel_y) = (cursor_x, cursor_y);
        let input_w = (max_inner_w - input_rel_x).max(min_input_w);
        let cnt_h = self
            .max_chars
            .map_or(0.0, |_| tfm.counter_row_height.points());
        let total_h = (input_rel_y + pill_h + pad_v * 2.0 + cnt_h).max(tfm.min_height.points());

        let alloc_sz = Vec2::new(width, total_h);
        let (raw_rect, response) = ui.allocate_exact_size(alloc_sz, Sense::click());
        let rect = subpixel_snap_rect(raw_rect, ui.ctx().pixels_per_point());
        if response.clicked() && self.enabled {
            ui.memory_mut(|m| m.request_focus(input_id));
        }

        let in_x = rect.left() + pad_h + input_rel_x;
        let in_y = rect.top() + pad_v + input_rel_y;
        let input_rect = Rect::from_min_size(Pos2::new(in_x, in_y), Vec2::new(input_w, pill_h));
        let txt_col = if self.enabled {
            theme.label()
        } else {
            theme.placeholder_text()
        };
        let mut child_ui = ui.new_child(
            UiBuilder::new()
                .id_salt((self.id, "token_field_inner"))
                .max_rect(input_rect),
        );
        let edit_w = TextEdit::singleline(self.input_buffer)
            .id(input_id)
            .font(font_id.clone())
            .text_color(txt_col)
            .frame(egui::Frame::NONE)
            .interactive(self.enabled);
        let edit_resp = child_ui.add_sized(input_rect.size(), edit_w);

        if edit_resp.changed() && self.input_buffer.contains(',') {
            for part in std::mem::take(self.input_buffer).split(',') {
                if !part.trim().is_empty() {
                    self.tokens.push(part.trim().to_string());
                    changed = true;
                }
            }
        }

        let has_mem_focus = ui.memory(|m| m.has_focus(input_id));
        let is_focused = edit_resp.has_focus() || response.has_focus() || has_mem_focus;
        let painter = ui.painter_at(rect);
        let bezel_r = tfm.corner_radius.points();

        if self.enabled && is_focused {
            theme.paint_focus_ring(&painter, rect, bezel_r);
        }

        let border_col = if is_focused {
            theme.control_accent()
        } else {
            theme.separator()
        };
        let border_s = Stroke::new(tfm.border_width.points(), border_col);
        let bezel_cr = CornerRadius::same(bezel_r as u8);
        let ctrl_bg = theme.control_background();
        painter.rect(rect, bezel_cr, ctrl_bg, border_s, StrokeKind::Inside);

        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let sel_fg = theme
            .selected_control_background_style()
            .primary_text_color(theme);
        let pill_border = Stroke::new(tfm.border_width.points(), theme.separator());
        let span = tfm.remove_cross_span.points();
        let rm_size_vec = Vec2::splat(remove_sz);
        let align_lc = Align2::LEFT_CENTER;
        let mut remove_idx = None;

        for &(idx, rel_x, rel_y, w) in &pill_layouts {
            if idx >= self.tokens.len() {
                continue;
            }
            let pill_pos = Pos2::new(rect.left() + pad_h + rel_x, rect.top() + pad_v + rel_y);
            let pill_rect = Rect::from_min_size(pill_pos, Vec2::new(w, pill_h));
            let rm_x = pill_rect.right() - pill_pad_h * 0.5 - remove_sz * 0.5;
            let rm_rect =
                Rect::from_center_size(Pos2::new(rm_x, pill_rect.center().y), rm_size_vec);
            let rm_resp = ui.interact(rm_rect, self.id.with(("rm_tok", idx)), sense);
            if rm_resp.clicked() && self.enabled {
                remove_idx = Some(idx);
            }
            let body_max = Pos2::new(rm_rect.left().max(pill_rect.left()), pill_rect.max.y);
            let body_rect = Rect::from_min_max(pill_rect.min, body_max);
            let body_resp = ui.interact(body_rect, self.id.with(("pill_body", idx)), sense);
            if body_resp.clicked() && self.enabled {
                selected_tok = Some(idx);
            }

            let (pill_bg, pill_fg, x_fg) = if selected_tok == Some(idx) && self.enabled {
                (theme.control_accent(), sel_fg, sel_fg)
            } else if rm_resp.hovered() && self.enabled {
                (theme.selected_control(), sel_fg, sel_fg)
            } else {
                (theme.control(), theme.label(), theme.secondary_label())
            };
            painter.rect(pill_rect, pill_r, pill_bg, pill_border, StrokeKind::Inside);

            let clip_r = (rm_rect.left() - 2.0).max(pill_rect.left());
            let text_clip = Rect::from_min_max(pill_rect.min, Pos2::new(clip_r, pill_rect.max.y));
            let text_pos = Pos2::new(pill_rect.left() + pill_pad_h, pill_rect.center().y);
            let clip_p = painter.with_clip_rect(text_clip);
            let tok_str = &self.tokens[idx];
            clip_p.text(text_pos, align_lc, tok_str, font_id.clone(), pill_fg);

            let c = rm_rect.center();
            let x_stroke = Stroke::new(tfm.remove_icon_stroke.points(), x_fg);
            let (x0, x1, y0, y1) = (c.x - span, c.x + span, c.y - span, c.y + span);
            painter.line_segment([Pos2::new(x0, y0), Pos2::new(x1, y1)], x_stroke);
            painter.line_segment([Pos2::new(x0, y1), Pos2::new(x1, y0)], x_stroke);
        }

        if let Some(idx) = remove_idx.filter(|&i| i < self.tokens.len()) {
            self.tokens.remove(idx);
            selected_tok = None;
            changed = true;
        }
        ui.data_mut(|d| d.insert_temp(sel_id, selected_tok.filter(|&i| i < self.tokens.len())));

        if self.tokens.is_empty() && self.input_buffer.is_empty() && !is_focused {
            let ph_pos = Pos2::new(input_rect.left(), input_rect.center().y);
            painter.text(
                ph_pos,
                align_lc,
                self.placeholder,
                font_id,
                theme.placeholder_text(),
            );
        }

        if let Some(max_c) = self.max_chars {
            let tok_c: usize = self.tokens.iter().map(|t| t.chars().count()).sum();
            let total_c = tok_c + self.input_buffer.chars().count();
            let col = if total_c > max_c {
                theme.system_red()
            } else {
                theme.secondary_label()
            };
            let cnt_pos = Pos2::new(rect.right() - pad_h, rect.bottom() - pad_v * 0.5);
            let cnt_f = FontId::proportional(tfm.counter_font_size.points());
            let cnt_txt = format!("{total_c} / {max_c}");
            painter.text(cnt_pos, Align2::RIGHT_BOTTOM, cnt_txt, cnt_f, col);
        }

        let mut final_resp = edit_resp.union(response);
        if changed {
            final_resp.mark_changed();
        }
        final_resp
    }
}
