//! State 2 (`v0.3`): Cupertino Symbols & Layout Lab (`GalleryScreen::VectorAssets`).

use super::{CenterState, SYMBOLS_META};
use crate::CardModel;
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui,
};
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacColorWell, MacFixedView, MacRow, MacScrollView,
    MacSegmentedControl, MacSlider, MacToastManager, MacToggle,
};
use std::cell::RefCell;

impl CenterState {
    pub fn render_state2_symbols_layout(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new(
                "State 2 (v0.3) — Cupertino Symbols & Layout Engines Lab (MacSymbol, MacFixedView, MacRow, MacScrollView)",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.add_space(6.0);

        content_ui.horizontal(|h| {
            h.label(egui::RichText::new("Glyph Scale:").size(11.0).strong());
            MacSegmentedControl::new(&["16 pt", "24 pt", "32 pt"], self.symbol_preview_size_idx)
                .theme(theme)
                .height(22.0)
                .show(h, |idx| {
                    self.symbol_preview_size_idx = idx;
                });
            h.add_space(12.0);
            h.label(egui::RichText::new("Symbol Tint:").size(11.0).strong());
            MacColorWell::new(&mut self.symbol_tint)
                .theme(theme)
                .show(h);
        });
        content_ui.add_space(6.0);

        let glyph_pt = match self.symbol_preview_size_idx {
            0 => 16.0,
            2 => 32.0,
            _ => 24.0,
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

        let item_w = 88.0f32;
        let item_h = 72.0f32;
        let spacing = 8.0f32;
        let avail_w = content_ui.available_width().max(300.0);
        let cols = ((avail_w + spacing) / (item_w + spacing)).floor().max(2.0) as usize;

        egui::Grid::new("vector_symbols_grid")
            .spacing(vec2(spacing, spacing))
            .show(content_ui, |grid_ui| {
                for (col_idx, (sym, name, _, cp_hex)) in filtered.iter().enumerate() {
                    let is_selected = *sym == self.selected_symbol;
                    let (rect, resp) =
                        grid_ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
                    if resp.clicked() {
                        self.selected_symbol = *sym;
                        toasts
                            .borrow_mut()
                            .info(format!("Selected {name} ({cp_hex})"));
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
                            if is_selected {
                                self.symbol_tint
                            } else {
                                theme.separator()
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    p.text(
                        pos2(rect.center().x, rect.min.y + 26.0),
                        Align2::CENTER_CENTER,
                        sym.as_str(),
                        FontId::proportional(glyph_pt),
                        if is_selected {
                            self.symbol_tint
                        } else {
                            theme.label()
                        },
                    );
                    p.text(
                        pos2(rect.center().x, rect.max.y - 12.0),
                        Align2::CENTER_CENTER,
                        *name,
                        FontId::proportional(10.0),
                        theme.secondary_label(),
                    );
                    if (col_idx + 1) % cols == 0 {
                        grid_ui.end_row();
                    }
                }
            });

        content_ui.add_space(14.0);
        content_ui.label(
            egui::RichText::new("MacFixedView — Hardware Clipping & Absolute Point Placement")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.add_space(4.0);
        MacSegmentedControl::new(
            &["Vector Card", "Raster Mask", "Core Animation"],
            self.fixed_seg_idx,
        )
        .theme(theme)
        .height(24.0)
        .show(content_ui, |idx| {
            self.fixed_seg_idx = idx;
        });
        content_ui.add_space(6.0);

        let fixed_w = (content_ui.available_width() - 32.0).max(320.0);
        let fixed_h = 126.0;
        let (fixed_container_rect, _) =
            content_ui.allocate_exact_size(vec2(fixed_w, fixed_h), Sense::hover());
        let fp = content_ui.painter_at(fixed_container_rect);
        fp.rect(
            fixed_container_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let cur_time = content_ui.input(|i| i.time);
        match self.fixed_seg_idx {
            0 => {
                let mut fixed_view = MacFixedView::new(content_ui, fixed_container_rect);
                let action_label = format!("Vector Action A ({})", self.vector_action_count);
                let clicked_a = fixed_view.subview(pos2(14.0, 14.0), vec2(155.0, 32.0), |ui_a| {
                    ui_a.add(MacButtonCell::new(&action_label).theme(theme))
                        .clicked()
                });
                if clicked_a {
                    self.vector_action_count += 1;
                    card.rotation_deg = (card.rotation_deg + 15.0) % 360.0;
                    toasts.borrow_mut().success(format!(
                        "Vector Action A #{} (Rotated card +15°)",
                        self.vector_action_count
                    ));
                }

                let prev_snap = self.fixed_grid_snap;
                fixed_view.subview(pos2(180.0, 14.0), vec2(160.0, 32.0), |ui_b| {
                    ui_b.horizontal(|h| {
                        h.label(egui::RichText::new("Grid Snap:").size(11.0));
                        h.add(MacToggle::new(&mut self.fixed_grid_snap).theme(theme));
                    });
                });
                if self.fixed_grid_snap != prev_snap {
                    let snap_step = if self.fixed_grid_snap { 16.0 } else { 1.0 };
                    let w_num = card.width.parse::<f32>().unwrap_or(260.0);
                    card.width = format!("{:.1}", (w_num / snap_step).round() * snap_step);
                    toasts.borrow_mut().info(if self.fixed_grid_snap {
                        "16pt Grid Snap applied"
                    } else {
                        "Free subpixel placement"
                    });
                }

                fixed_view.subview(pos2(14.0, 62.0), vec2(360.0, 48.0), |ui_c| {
                    ui_c.label(
                        egui::RichText::new(format!(
                            "Active Card Width: {} pt | Grid Snap: {} | Action Clicks: {}",
                            card.width,
                            if self.fixed_grid_snap { "16pt" } else { "Off" },
                            self.vector_action_count
                        ))
                        .size(10.5)
                        .color(theme.secondary_label()),
                    );
                });
            }
            1 => {
                let clip_rect = fixed_container_rect.shrink2(vec2(24.0, 16.0));
                let clipped_painter = content_ui.painter().with_clip_rect(clip_rect);
                clipped_painter.rect_filled(
                    clip_rect,
                    CornerRadius::same(6),
                    Color32::from_rgb(22, 24, 28),
                );
                let mut sx = clip_rect.min.x - 60.0;
                while sx < clip_rect.max.x + 60.0 {
                    clipped_painter.line_segment(
                        [
                            pos2(sx, clip_rect.max.y + 20.0),
                            pos2(sx + 50.0, clip_rect.min.y - 20.0),
                        ],
                        Stroke::new(14.0, theme.control_accent().gamma_multiply(0.45)),
                    );
                    sx += 36.0;
                }
                clipped_painter.text(
                    clip_rect.center(),
                    Align2::CENTER_CENTER,
                    "Hardware Raster Mask — set_clip_rect()",
                    FontId::proportional(11.0),
                    Color32::WHITE,
                );
            }
            _ => {
                let mut fixed_view = MacFixedView::new(content_ui, fixed_container_rect);
                let clicked = fixed_view.subview(pos2(14.0, 12.0), vec2(240.0, 24.0), |btn_ui| {
                    btn_ui
                        .add(MacButtonCell::new("Trigger Cascading Spring").theme(theme))
                        .clicked()
                });
                if clicked {
                    self.cascade_start_time = Some(cur_time);
                }
                for idx in 0..3 {
                    let delay = idx as f32 * 0.08;
                    let (disp_y, alpha) = if let Some(start_t) = self.cascade_start_time {
                        let elapsed = (cur_time - start_t) as f32;
                        if elapsed < self.bounce_spring.settling_duration() + 0.3 {
                            fixed_view.ui.ctx().request_repaint_after_secs(1.0 / 60.0);
                        }
                        let p = self.bounce_spring.evaluate((elapsed - delay).max(0.0));
                        ((1.0 - p) * 20.0, p.clamp(0.0, 1.0))
                    } else {
                        (0.0, 1.0)
                    };
                    let c_rect = Rect::from_min_size(
                        pos2(
                            fixed_container_rect.min.x + 14.0 + idx as f32 * 102.0,
                            fixed_container_rect.min.y + 46.0 + disp_y,
                        ),
                        vec2(88.0, 56.0),
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

        content_ui.add_space(12.0);
        content_ui.horizontal(|h| {
            h.label(egui::RichText::new("MacRow Budget:").size(11.0).strong());
            MacSlider::new(&mut self.row_container_width, 240.0..=620.0)
                .theme(theme)
                .width(180.0)
                .show(h);
            h.label(egui::RichText::new(format!("{:.0} pt", self.row_container_width)).size(11.0));
        });
        let (row_alloc_rect, _) =
            content_ui.allocate_exact_size(vec2(self.row_container_width, 32.0), Sense::hover());
        let rp = content_ui.painter_at(row_alloc_rect);
        let mut row = MacRow::new(
            row_alloc_rect.min.y + 2.0,
            28.0,
            self.row_container_width,
            6.0,
        );
        let (x0, y0, w0, h0) = row.allocate_cell(90.0);
        let (x1, y1, w1, h1) = row.allocate_cell(120.0);
        let rem_w = row.remaining_width().max(30.0);
        let (x2, y2, w2, h2) = row.allocate_cell(rem_w);
        for (x, y, w, h, col, txt) in [
            (
                x0,
                y0,
                w0,
                h0,
                Color32::from_rgb(30, 80, 160),
                "90pt".to_string(),
            ),
            (
                x1,
                y1,
                w1,
                h1,
                Color32::from_rgb(110, 50, 160),
                "120pt".to_string(),
            ),
            (
                x2,
                y2,
                w2,
                h2,
                Color32::from_rgb(20, 140, 110),
                format!("Flex {:.0}pt", w2),
            ),
        ] {
            let r = Rect::from_min_size(pos2(row_alloc_rect.min.x + x, y), vec2(w, h));
            rp.rect_filled(r, CornerRadius::same(4), col);
            rp.text(
                r.center(),
                Align2::CENTER_CENTER,
                txt,
                FontId::proportional(10.5),
                Color32::WHITE,
            );
        }

        content_ui.add_space(12.0);
        content_ui.label(
            egui::RichText::new(
                "MacScrollView (7pt → 10pt Hover Expansion & Track-Click Page Jump)",
            )
            .font(label_font)
            .strong(),
        );
        MacScrollView::both("demo_nested_scroll")
            .theme(theme)
            .auto_hide(false)
            .with_offset(&mut self.demo_scroll_offset)
            .max_width(fixed_w)
            .max_height(78.0)
            .show(content_ui, |sc_ui| {
                for r in 1..=6 {
                    sc_ui.label(
                        egui::RichText::new(format!(
                            "Scroll Row #{r}: Hover scrollbar track to expand 7pt → 10pt, or click empty track to page-jump."
                        ))
                        .size(10.5),
                    );
                }
            });
        content_ui.add_space(18.0);
    }
}
