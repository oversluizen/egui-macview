//! State 3 (`v0.4–v0.4.1`): Command, Menu & State Integrity Lab (`GalleryScreen::DesignTokensWcag`).

use super::CenterState;
use crate::CardModel;
use egui::{
    vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Layout, Sense, Stroke, StrokeKind, Ui,
};
use egui_macview::{
    contrast_ratio, relative_luminance, theme::MacTheme, MacBackgroundStyle, MacButtonCell,
    MacColorWell, MacControlSize, MacInputCell, MacPopUpButton, MacSegmentedControl, MacStepper,
    MacSymbol, MacToastManager, MacToggle, NSUndoManager,
};
use std::cell::RefCell;

impl CenterState {
    pub fn render_state3_command_state(
        &mut self,
        content_ui: &mut Ui,
        theme: &MacTheme,
        card: &mut CardModel,
        undo_manager: &RefCell<NSUndoManager<CardModel>>,
        last_stable_card: &RefCell<CardModel>,
        toasts: &RefCell<MacToastManager>,
    ) {
        let label_font = FontId::proportional(12.0);
        content_ui.label(
            egui::RichText::new(
                "State 3 (v0.4–v0.4.1) — Command, Menu, NSUndoManager & Control Size Integrity Lab",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.label(
            egui::RichText::new(
                "Demonstrates NSUndoManager drag grouping, MacControlSize × Enabled/Disabled matrix, numeric arrow stepping, and WCAG 2.1 AA contrast.",
            )
            .size(11.0),
        );
        content_ui.add_space(8.0);

        // 1. NSUndoManager & Micro-Feedback Toast Triggers
        content_ui.horizontal(|h| {
            let (can_undo, can_redo, undo_n, redo_m, grouping) = {
                let um = undo_manager.borrow();
                (
                    um.can_undo(),
                    um.can_redo(),
                    um.undo_count(),
                    um.redo_count(),
                    um.is_grouping(),
                )
            };
            let undo_lbl = format!("{} Undo ({undo_n})", MacSymbol::ArrowCounterClockwise.as_str());
            if h.add(MacButtonCell::new(&undo_lbl).theme(theme).enabled(can_undo))
                .clicked()
                && undo_manager.borrow_mut().undo(card)
            {
                *last_stable_card.borrow_mut() = card.clone();
                toasts.borrow_mut().info("NSUndoManager: Undone");
            }
            let redo_lbl = format!("{} Redo ({redo_m})", MacSymbol::ArrowClockwise.as_str());
            if h.add(MacButtonCell::new(&redo_lbl).theme(theme).enabled(can_redo))
                .clicked()
                && undo_manager.borrow_mut().redo(card)
            {
                *last_stable_card.borrow_mut() = card.clone();
                toasts.borrow_mut().info("NSUndoManager: Redone");
            }
            if h.add(MacButtonCell::new("Toast: Info").theme(theme)).clicked() {
                toasts.borrow_mut().info("AppKit HUD Notification");
            }
            if h.add(MacButtonCell::new("Toast: Success").theme(theme)).clicked() {
                toasts.borrow_mut().success("State Snapshot Verified");
            }
            if h.add(MacButtonCell::new("Toast: Warning").theme(theme)).clicked() {
                toasts.borrow_mut().warning("Contrast Threshold Alert");
            }
            h.label(
                egui::RichText::new(if grouping {
                    "Group: Active"
                } else {
                    "Group: Idle"
                })
                .size(10.5)
                .color(theme.secondary_label()),
            );
        });

        // 2. MacControlSize × Enabled/Disabled Comparison Matrix + Numeric Stepping
        content_ui.add_space(12.0);
        content_ui.label(
            egui::RichText::new(
                "NSControlSize Matrix (Mini / Compact / Small / Regular / Large) & Numeric Stepping",
            )
            .font(label_font.clone())
            .strong(),
        );
        content_ui.add_space(4.0);

        content_ui.horizontal(|h| {
            h.label(egui::RichText::new("Size Tier:").size(11.0).strong());
            MacSegmentedControl::new(
                &[
                    "Mini (15pt)",
                    "Compact (18pt)",
                    "Small (19pt)",
                    "Regular (22pt)",
                    "Large (28pt)",
                ],
                self.showcase_control_size_idx,
            )
            .theme(theme)
            .height(22.0)
            .show(h, |idx| {
                self.showcase_control_size_idx = idx;
            });
            h.add_space(8.0);
            h.label(egui::RichText::new("Enabled:").size(11.0).strong());
            h.add(MacToggle::new(&mut self.showcase_controls_enabled).theme(theme));
        });
        content_ui.add_space(6.0);

        let active_ctrl_size = match self.showcase_control_size_idx {
            0 => MacControlSize::Mini,
            1 => MacControlSize::CompactInfobox,
            2 => MacControlSize::Small,
            3 => MacControlSize::Regular,
            _ => MacControlSize::Large,
        };
        let is_en = self.showcase_controls_enabled;

        content_ui.horizontal(|h| {
            MacInputCell::with_id(Id::NULL.with("showcase_input_cell"), &mut self.showcase_text)
                .theme(theme)
                .control_size(active_ctrl_size)
                .width(110.0)
                .enabled(is_en)
                .show(h);

            h.label(egui::RichText::new("Numeric ↑/↓:").size(11.0));
            MacInputCell::with_id(
                Id::NULL.with("showcase_num_input_cell"),
                &mut self.showcase_num_text,
            )
            .theme(theme)
            .control_size(active_ctrl_size)
            .numeric(0.0..=1000.0, 1.0)
            .width(68.0)
            .enabled(is_en)
            .show(h);

            MacStepper::new(&mut self.showcase_step_val, -1000.0..=1000.0, 1.0)
                .theme(theme)
                .enabled(is_en)
                .show(h);
            h.label(egui::RichText::new(format!("{:.0}", self.showcase_step_val)).size(11.0).strong());

            MacPopUpButton::new(
                &mut self.showcase_popup_idx,
                &["Aqua Preset", "Graphite", "Vibrant Pro"],
            )
            .theme(theme)
            .width(110.0)
            .enabled(is_en)
            .show(h);

            MacColorWell::new(&mut self.showcase_color)
                .theme(theme)
                .enabled(is_en)
                .show(h);
        });

        // 3. Real-Time WCAG 2.1 AA Contrast Engine
        content_ui.add_space(14.0);
        content_ui.label(
            egui::RichText::new("Real-Time WCAG 2.1 AA Contrast Evaluation Engine (L ≤ 0.179)")
                .font(label_font.clone())
                .strong(),
        );
        content_ui.add_space(4.0);

        let card_w = (content_ui.available_width() - 32.0).max(300.0);
        let (eval_rect, _) = content_ui.allocate_exact_size(vec2(card_w, 132.0), Sense::hover());
        let ep = content_ui.painter_at(eval_rect);
        ep.rect(
            eval_rect,
            CornerRadius::same(8),
            theme.window_background(),
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );

        let mut eval_ui = content_ui.new_child(
            egui::UiBuilder::new()
                .max_rect(eval_rect.shrink(10.0))
                .layout(Layout::top_down(Align::Min)),
        );
        eval_ui.horizontal(|h| {
            h.label(egui::RichText::new("FG:").size(11.0).strong());
            h.add(MacColorWell::new(&mut self.wcag_fg).theme(theme));
            h.add_space(8.0);
            h.label(egui::RichText::new("BG:").size(11.0).strong());
            h.add(MacColorWell::new(&mut self.wcag_bg).theme(theme));
            h.add_space(10.0);
            if h.add(MacButtonCell::new("Label on Window").theme(theme)).clicked() {
                self.wcag_fg = theme.label();
                self.wcag_bg = theme.window_background();
            }
            if h.add(MacButtonCell::new("White on Accent").theme(theme)).clicked() {
                self.wcag_fg = Color32::WHITE;
                self.wcag_bg = theme.control_accent();
            }
            if h.add(MacButtonCell::new("Dark on Accent").theme(theme)).clicked() {
                self.wcag_fg = Color32::from_rgb(30, 30, 30);
                self.wcag_bg = theme.control_accent();
            }
        });
        eval_ui.add_space(6.0);

        let l_fg = relative_luminance(self.wcag_fg);
        let l_bg = relative_luminance(self.wcag_bg);
        let ratio = contrast_ratio(self.wcag_fg, self.wcag_bg);
        let bg_style = MacBackgroundStyle::from_color(self.wcag_bg);
        eval_ui.label(
            egui::RichText::new(format!(
                "L(fg)={l_fg:.4} | L(bg)={l_bg:.4} | Ratio={ratio:.2}:1 | Style={bg_style:?} (Normal AA: {})",
                if ratio >= 4.5 { "PASS ✓" } else { "FAIL ✗" }
            ))
            .size(11.0)
            .strong(),
        );
        eval_ui.add_space(6.0);
        let (sample_rect, _) =
            eval_ui.allocate_exact_size(vec2(card_w - 24.0, 28.0), Sense::hover());
        let sp = eval_ui.painter_at(sample_rect);
        sp.rect(
            sample_rect,
            CornerRadius::same(5),
            self.wcag_bg,
            Stroke::new(1.0, theme.separator()),
            StrokeKind::Inside,
        );
        sp.text(
            sample_rect.center(),
            Align2::CENTER_CENTER,
            "The quick brown fox jumps over the lazy dog (WCAG 2.1 AA Sample)",
            FontId::proportional(11.5),
            self.wcag_fg,
        );

        // 4. W3C DTCG Token Catalogue
        content_ui.add_space(12.0);
        MacSegmentedControl::new(
            &["Control Metrics", "Semantic Palettes", "Motion Parameters"],
            self.token_tab_idx,
        )
        .theme(theme)
        .height(22.0)
        .show(content_ui, |idx| {
            self.token_tab_idx = idx;
        });
        content_ui.add_space(6.0);

        let q = self.search_query.trim().to_lowercase();
        let rows: &[(&str, &str)] = match self.token_tab_idx {
            0 => &[
                ("button.regular.height", "22.0 pt — Standard push button"),
                ("text_field.regular.height", "22.0 pt — Standard NSTextField"),
                ("outline_view.row_height", "24.0 pt — Sidebar row height"),
                ("toolbar.height_unified", "52.0 pt — Unified AppKit toolbar"),
                ("table_view.header_height", "28.0 pt — NSTableHeaderView height"),
            ],
            1 => &[
                ("window_background", "Dynamic Aqua / DarkAqua surface"),
                ("control_accent", "System accent blue (#007AFF / #0A84FF)"),
                ("keyboard_focus_indicator", "50% alpha outer focus ring halo"),
            ],
            _ => &[
                ("motion.hud_panel_transition", "T₀ = 0.30s, ζ = 0.70"),
                ("motion.selection_spring", "T₀ = 0.20s, ζ = 0.85"),
            ],
        };
        for (k, v) in rows {
            if !q.is_empty() && !k.to_lowercase().contains(&q) && !v.to_lowercase().contains(&q) {
                continue;
            }
            content_ui.horizontal(|h| {
                h.label(egui::RichText::new(*k).size(11.0).monospace().color(theme.control_accent()));
                h.label(egui::RichText::new(*v).size(10.5).color(theme.secondary_label()));
            });
        }
        content_ui.add_space(18.0);
    }
}
