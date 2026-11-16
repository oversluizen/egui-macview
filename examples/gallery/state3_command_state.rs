//! State 3 (`v0.4–v0.4.1`): Command, Menu & State Integrity Lab (`GalleryScreen::DesignTokensWcag`).

use super::CenterState;
use egui::{vec2, Align, Align2, CornerRadius, FontId, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder};
use egui_macview::{contrast_ratio, relative_luminance, theme::MacTheme, MacBackgroundStyle};

impl CenterState {
    pub fn render_wcag_contrast_sample_slot(&mut self, content_ui: &mut Ui, theme: &MacTheme, slot_rect: Rect) {
        let mut eval_ui = content_ui
            .new_child(UiBuilder::new().id_salt("gallery_wcag_eval_slot_ui").max_rect(slot_rect).layout(Layout::top_down(Align::Min)));
        let l_fg = relative_luminance(self.wcag_fg);
        let l_bg = relative_luminance(self.wcag_bg);
        let ratio = contrast_ratio(self.wcag_fg, self.wcag_bg);
        let bg_style = MacBackgroundStyle::from_color(self.wcag_bg);
        let target = if self.high_contrast { self.contrast_target.max(7.0) } else { self.contrast_target };
        let target_pass = if ratio >= target { "PASS" } else { "FAIL" };
        let ui_pass = if ratio >= 3.0 { "PASS" } else { "FAIL" };
        eval_ui.label(
            egui::RichText::new(format!(
                "L(fg)={l_fg:.4} | L(bg)={l_bg:.4} | Ratio={ratio:.2}:1 | Target (>={target:.1}:1): {target_pass} | UI AA: {ui_pass} | Style={bg_style:?}"
            ))
            .size(11.0)
            .strong(),
        );
        eval_ui.add_space(4.0);
        let (sample_rect, _) = eval_ui.allocate_exact_size(vec2(slot_rect.width(), 30.0), Sense::hover());
        let sp = eval_ui.painter_at(sample_rect);
        sp.rect(
            sample_rect,
            CornerRadius::same(5),
            self.wcag_bg,
            Stroke::new(if self.high_contrast { 2.0 } else { 1.0 }, theme.separator()),
            StrokeKind::Inside,
        );
        let codec_lbl = ["Apple ProRes", "H.264", "HEVC"][self.showcase_popup_idx.min(2)];
        sp.text(
            sample_rect.center(),
            Align2::CENTER_CENTER,
            format!(
                "WCAG 2.1 Sample — {} | {codec_lbl} | Slider: {:.0}% | ReduceTransp: {}",
                self.showcase_text,
                self.matrix_slider,
                if self.reduce_transparency { "ON" } else { "OFF" }
            ),
            FontId::proportional(11.5),
            self.wcag_fg,
        );
        eval_ui.add_space(6.0);
        let q = self.search_query.trim().to_lowercase();
        for (k, v) in [
            ("button.regular.height", "22.0 pt — Standard push button"),
            ("text_field.regular.height", "22.0 pt — Standard NSTextField"),
            ("table_view.header_height", "28.0 pt — NSTableHeaderView height"),
        ] {
            if !q.is_empty() && !k.to_lowercase().contains(&q) && !v.to_lowercase().contains(&q) {
                continue;
            }
            eval_ui.horizontal(|h| {
                h.label(egui::RichText::new(k).size(10.5).monospace().color(theme.control_accent()));
                h.label(egui::RichText::new(v).size(10.0).color(theme.secondary_label()));
            });
        }
    }
}
