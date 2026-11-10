//! Auxiliary publish queue controls and progress indicators for State 4 Finder (`SPEC-011`).

use super::CenterState;
use egui::Ui;
use egui_macview::{
    theme::MacTheme, MacButtonCell, MacProgressControlSize, MacProgressIndicator, MacSlider,
    MacToastManager, MacToggle,
};
use std::cell::RefCell;

impl CenterState {
    /// Renders the MacProgressIndicator publish queue, scrub slider, and indeterminate controls.
    pub fn render_state4_publish_queue(
        &mut self,
        ui: &mut Ui,
        theme: &MacTheme,
        toasts: &RefCell<MacToastManager>,
        window_active: bool,
        t: f64,
    ) {
        ui.add_space(10.0);
        ui.horizontal(|h_ui| {
            let btn_label = if self.is_publishing {
                "Pause Publish"
            } else {
                "Publish Selected Asset"
            };
            if h_ui
                .add(
                    MacButtonCell::new(btn_label)
                        .theme(theme)
                        .selected(self.is_publishing),
                )
                .clicked()
            {
                if !self.is_publishing && self.publish_progress >= 1.0 {
                    self.publish_progress = 0.0;
                }
                self.is_publishing = !self.is_publishing;
                toasts.borrow_mut().info(if self.is_publishing {
                    "Publishing started..."
                } else {
                    "Publishing paused"
                });
            }

            let mut bar = MacProgressIndicator::bar(self.publish_progress)
                .theme(theme)
                .control_size(MacProgressControlSize::Regular)
                .show_percentage(true)
                .width(180.0);
            if self.publish_indeterminate {
                let indet_t = if window_active {
                    (t * 0.65).fract()
                } else {
                    0.0
                };
                bar = bar.indeterminate(indet_t);
            }
            bar.show(h_ui);

            let spin_t = if (self.is_publishing || self.publish_indeterminate) && window_active {
                t.fract().max(1e-4)
            } else {
                0.0
            };
            MacProgressIndicator::spinning(spin_t)
                .theme(theme)
                .control_size(MacProgressControlSize::Regular)
                .show(h_ui);
        });

        ui.add_space(6.0);
        ui.horizontal(|h_ui| {
            h_ui.label(egui::RichText::new("Manual Scrub:").size(11.0));
            MacSlider::new(&mut self.publish_progress, 0.0..=1.0)
                .theme(theme)
                .width(140.0)
                .show(h_ui);
            h_ui.add_space(8.0);
            h_ui.label(egui::RichText::new("Indeterminate:").size(11.0));
            h_ui.add(MacToggle::new(&mut self.publish_indeterminate).theme(theme));
        });
        ui.add_space(14.0);
    }
}
