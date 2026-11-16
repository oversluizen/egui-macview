//! State 2 (`v0.3`): Cupertino Symbols & Layout Lab (`GalleryScreen::VectorAssets`).

use super::{CenterState, SYMBOLS_META};
use crate::CardModel;
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder};
use egui_macview::{theme::MacTheme, MacFixedView, MacRow, MacScrollView, MacToastManager};
use std::cell::RefCell;

impl CenterState {
    pub fn render_symbol_grid_slot(&mut self, content_ui: &mut Ui, theme: &MacTheme, toasts: &RefCell<MacToastManager>, slot_rect: Rect) {
        let glyph_pt = match self.symbol_preview_size_idx {
            0 => 14.0,
            2 => 28.0,
            _ => 20.0,
        };
        let q = self.search_query.trim().to_lowercase();
        let filtered: Vec<_> = SYMBOLS_META
            .iter()
            .copied()
            .filter(|(sym, name, usage, cp)| {
                q.is_empty()
                    || name.to_lowercase().contains(&q)
                    || usage.to_lowercase().contains(&q)
                    || cp.to_lowercase().contains(&q)
                    || format!("{:?}", sym).to_lowercase().contains(&q)
            })
            .collect();

        let item_w = 92.0f32;
        let item_h = 56.0f32;
        let spacing = 6.0f32;
        let cols = ((slot_rect.width() + spacing) / (item_w + spacing)).floor().max(2.0) as usize;

        let mut grid_host = content_ui
            .new_child(UiBuilder::new().id_salt("state2_symbol_grid_slot").max_rect(slot_rect).layout(Layout::top_down(Align::Min)));
        MacScrollView::new("state2_symbol_grid_scroll").theme(theme).max_width(slot_rect.width()).max_height(slot_rect.height()).show(
            &mut grid_host,
            |scroll_ui| {
                egui::Grid::new("vector_symbols_grid").spacing(vec2(spacing, spacing)).show(scroll_ui, |grid_ui| {
                    for (col_idx, (sym, name, _, cp_hex)) in filtered.iter().enumerate() {
                        let is_selected = *sym == self.selected_symbol;
                        let (rect, resp) = grid_ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
                        if resp.clicked() {
                            self.selected_symbol = *sym;
                            toasts.borrow_mut().info(format!("Selected {name} ({cp_hex})"));
                        }
                        let p = grid_ui.painter_at(rect);
                        let bg = if is_selected {
                            self.symbol_tint.gamma_multiply(0.18)
                        } else if resp.hovered() {
                            theme.control().gamma_multiply(0.6)
                        } else {
                            theme.window_background()
                        };
                        p.rect(
                            rect,
                            CornerRadius::same(6),
                            bg,
                            Stroke::new(
                                if is_selected { 1.5 } else { 1.0 },
                                if is_selected { self.symbol_tint } else { theme.separator() },
                            ),
                            StrokeKind::Inside,
                        );
                        p.text(
                            pos2(rect.center().x, rect.min.y + 20.0),
                            Align2::CENTER_CENTER,
                            sym.as_str(),
                            FontId::proportional(glyph_pt),
                            if is_selected { self.symbol_tint } else { theme.label() },
                        );
                        p.text(
                            pos2(rect.center().x, rect.max.y - 10.0),
                            Align2::CENTER_CENTER,
                            *name,
                            FontId::proportional(9.5),
                            theme.secondary_label(),
                        );
                        if (col_idx + 1) % cols == 0 {
                            grid_ui.end_row();
                        }
                    }
                });
            },
        );
    }

    pub fn render_fixed_clipping_slot(&mut self, content_ui: &mut Ui, theme: &MacTheme, card: &CardModel, slot_rect: Rect) {
        let fp = content_ui.painter_at(slot_rect);
        fp.rect(slot_rect, CornerRadius::same(8), theme.window_background(), Stroke::new(1.0, theme.separator()), StrokeKind::Inside);

        let cur_time = content_ui.input(|i| i.time);
        if self.cascade_start_time.is_some_and(|t| t < 0.0) {
            self.cascade_start_time = Some(cur_time);
        }
        if self.fixed_seg_idx == 1 {
            let clip_rect = slot_rect.shrink2(vec2(16.0, 12.0));
            let clipped_painter = content_ui.painter().with_clip_rect(clip_rect);
            clipped_painter.rect_filled(clip_rect, CornerRadius::same(6), Color32::from_rgb(22, 24, 28));
            let mut sx = clip_rect.min.x - 40.0;
            while sx < clip_rect.max.x + 40.0 {
                clipped_painter.line_segment(
                    [pos2(sx, clip_rect.max.y + 16.0), pos2(sx + 40.0, clip_rect.min.y - 16.0)],
                    Stroke::new(12.0, theme.control_accent().gamma_multiply(0.45)),
                );
                sx += 32.0;
            }
            clipped_painter.text(
                clip_rect.center(),
                Align2::CENTER_CENTER,
                "Overflow Unclipped — Content Extends Beyond Bounds",
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        } else {
            let mut fixed_view = MacFixedView::new(content_ui, slot_rect);
            fixed_view.subview(pos2(12.0, 10.0), vec2(280.0, 26.0), |ui_c| {
                ui_c.label(
                    egui::RichText::new(format!(
                        "Card W: {}pt | Snap: {} | Actions: {}",
                        card.width,
                        if self.fixed_grid_snap { "16pt" } else { "Off" },
                        self.vector_action_count
                    ))
                    .size(10.5)
                    .color(theme.secondary_label()),
                );
            });
            for idx in 0..3 {
                let delay = idx as f32 * 0.08;
                let (disp_y, alpha) = if let Some(start_t) = self.cascade_start_time {
                    let elapsed = (cur_time - start_t).max(0.0) as f32;
                    if elapsed < self.bounce_spring.settling_duration() + 0.3 {
                        fixed_view.ui.ctx().request_repaint_after_secs(1.0 / 60.0);
                    }
                    let p = self.bounce_spring.evaluate((elapsed - delay).max(0.0));
                    ((1.0 - p) * 18.0, p.clamp(0.0, 1.0))
                } else {
                    (0.0, 1.0)
                };
                let c_rect = Rect::from_min_size(
                    pos2(slot_rect.min.x + 12.0 + idx as f32 * 96.0, slot_rect.min.y + 44.0 + disp_y),
                    vec2(84.0, 66.0),
                );
                fp.rect(
                    c_rect,
                    CornerRadius::same(6),
                    theme.control_accent().gamma_multiply(0.2 * alpha),
                    Stroke::new(1.0, theme.control_accent().gamma_multiply(alpha)),
                    StrokeKind::Inside,
                );
                fp.text(
                    c_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("Layer {}", idx + 1),
                    FontId::proportional(11.0),
                    theme.label().gamma_multiply(alpha),
                );
            }
        }
    }

    pub fn render_row_scroll_slot(&mut self, content_ui: &mut Ui, theme: &MacTheme, slot_rect: Rect) {
        let mut child_ui = content_ui
            .new_child(UiBuilder::new().id_salt("state2_row_scroll_slot").max_rect(slot_rect).layout(Layout::top_down(Align::Min)));
        let budget_w = self.row_container_width.clamp(220.0, slot_rect.width());
        let (row_alloc_rect, _) = child_ui.allocate_exact_size(vec2(budget_w, 28.0), Sense::hover());
        let rp = child_ui.painter_at(row_alloc_rect);
        let mut row = MacRow::new(row_alloc_rect.min.y + 2.0, 24.0, budget_w, 6.0);
        let (x0, y0, w0, h0) = row.allocate_cell(76.0);
        let (x1, y1, w1, h1) = row.allocate_cell(96.0);
        let rem_w = row.remaining_width().max(30.0);
        let (x2, y2, w2, h2) = row.allocate_cell(rem_w);
        for (x, y, w, h, col, txt) in [
            (x0, y0, w0, h0, Color32::from_rgb(30, 80, 160), "76pt".to_string()),
            (x1, y1, w1, h1, Color32::from_rgb(110, 50, 160), "96pt".to_string()),
            (x2, y2, w2, h2, Color32::from_rgb(20, 140, 110), format!("Flex {:.0}pt", w2)),
        ] {
            let r = Rect::from_min_size(pos2(row_alloc_rect.min.x + x, y), vec2(w, h));
            rp.rect_filled(r, CornerRadius::same(4), col);
            rp.text(r.center(), Align2::CENTER_CENTER, txt, FontId::proportional(10.0), Color32::WHITE);
        }
        child_ui.add_space(4.0);
        let font_name =
            ["Inter Regular (OFL)", "Inter SemiBold (OFL)", "Hack Monospace (MIT)", "Ubuntu Light (UFL)"][self.typo_font_family.min(3)];
        let weight_name = ["Regular", "Medium", "Semibold", "Bold", "Italic"][self.typo_font_weight.min(4)];
        let pt_sz = self.typo_point_size.clamp(9.0, 22.0);
        let is_mono = self.typo_font_family == 2;
        let is_ubuntu = self.typo_font_family == 3;
        let is_italic = self.typo_font_weight == 4;
        let use_semibold = self.typo_font_family == 1 || (!is_mono && !is_ubuntu && (1..=3).contains(&self.typo_font_weight));
        let font_id = if is_mono {
            FontId::monospace(pt_sz)
        } else if is_ubuntu {
            FontId::new(pt_sz, egui::FontFamily::Name("Ubuntu-Light".into()))
        } else if use_semibold {
            theme.semibold_font_id(&child_ui, pt_sz).0
        } else {
            FontId::proportional(pt_sz)
        };
        let (spec_rect, _) = child_ui.allocate_exact_size(vec2(slot_rect.width(), 38.0), Sense::hover());
        let sp = child_ui.painter_at(spec_rect);
        if self.show_grid {
            let mut gy = spec_rect.min.y + 6.0;
            while gy < spec_rect.max.y {
                sp.line_segment(
                    [pos2(spec_rect.min.x, gy), pos2(spec_rect.max.x, gy)],
                    Stroke::new(0.5, theme.control_accent().gamma_multiply(0.35)),
                );
                gy += 8.0;
            }
        }
        let sample_str = format!(
            "{font_name} {weight_name} ({:.0}pt, trk {:+.2}em, kern: {})",
            self.typo_point_size,
            self.typo_tracking_em,
            if self.typo_optical_kern { "ON" } else { "OFF" }
        );
        let mut fmt = theme.tracked_text_format(font_id.clone(), self.symbol_tint);
        fmt.extra_letter_spacing += self.typo_tracking_em * pt_sz;
        fmt.italics = is_italic;
        let galley = sp.layout_job(egui::text::LayoutJob::single_section(sample_str, fmt));
        let txt_pos = pos2(spec_rect.min.x + 4.0, spec_rect.min.y + 2.0);
        sp.galley(txt_pos, galley.clone(), self.symbol_tint);
        if self.typo_font_weight == 3 {
            sp.galley(txt_pos + vec2(0.7, 0.0), galley, self.symbol_tint);
        }
        let semi_id = theme.semibold_font_id(&child_ui, 11.0).0;
        let mut sx = spec_rect.min.x + 4.0;
        let sy = spec_rect.min.y + 22.0;
        for (lbl, fid, bold_pass, ital) in [
            ("Regular", FontId::proportional(11.0), false, false),
            ("Semibold", semi_id.clone(), false, false),
            ("Bold", semi_id, true, false),
            ("Mono", FontId::monospace(11.0), false, false),
            ("Italic", FontId::proportional(11.0), false, true),
        ] {
            let mut f = theme.tracked_text_format(fid, theme.label());
            f.italics = ital;
            let g = sp.layout_job(egui::text::LayoutJob::single_section(lbl.to_string(), f));
            let w = g.size().x;
            sp.galley(pos2(sx, sy), g.clone(), theme.label());
            if bold_pass {
                sp.galley(pos2(sx + 0.65, sy), g, theme.label());
            }
            sx += w + 14.0;
        }
        child_ui.add_space(2.0);
        MacScrollView::both("demo_nested_scroll")
            .theme(theme)
            .auto_hide(false)
            .with_offset(&mut self.demo_scroll_offset)
            .max_width(slot_rect.width())
            .max_height((slot_rect.height() - 68.0).max(48.0))
            .show(&mut child_ui, |sc_ui| {
                for r in 1..=6 {
                    sc_ui.label(egui::RichText::new(format!("Scroll Row #{r}: Hover scrollbar track to expand 7pt -> 10pt.")).size(10.5));
                }
            });
    }
}
