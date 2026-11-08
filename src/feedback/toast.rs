//! Authentic macOS Animated Toast Notifications.
//!
//! Normative Reference: SPEC-008B §4.
//! Floating pill banner with `CASpring` entry/exit physics and 2.5s auto-dismiss timer.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{Area, CornerRadius, FontId, Id, Order, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

static TOAST_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub use crate::tokens::DEFAULT_TOAST_DURATION;

/// Semantic styling variant of the toast pill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastLevel {
    /// Informational toast with neutral appearance.
    #[default]
    Info,
    /// Success toast with Apple system green accent.
    Success,
    /// Warning toast with system accent warning color.
    Warning,
    /// Error toast with crimson alert accent.
    Error,
}

/// Individual animated toast notification banner.
#[derive(Debug, Clone)]
pub struct MacToast {
    /// Unique identifier for this toast instance.
    pub id: String,
    /// Message string displayed in the pill.
    pub message: String,
    /// Severity level determining border and icon color.
    pub level: ToastLevel,
    /// Total duration in seconds before auto-dismissal.
    pub duration: f32,
    /// Elapsed seconds since presentation.
    pub elapsed: f32,
    /// CoreAnimation spring physics driver for entry/exit animation.
    pub spring: CASpring,
    /// Whether the toast has completed display and is dismissed.
    pub is_dismissed: bool,
}

impl MacToast {
    /// Creates a new toast notification with the standard 2.5s auto-dismiss duration.
    pub fn new(message: impl Into<String>) -> Self {
        let default_theme = MacTheme::default();
        Self {
            id: TOAST_ID_COUNTER.fetch_add(1, Ordering::Relaxed).to_string(),
            message: message.into(),
            level: ToastLevel::Info,
            duration: DEFAULT_TOAST_DURATION,
            elapsed: 0.0,
            spring: default_theme.motion().caspring_default.to_spring(),
            is_dismissed: false,
        }
    }

    /// Sets semantic severity level.
    pub fn level(mut self, level: ToastLevel) -> Self {
        self.level = level;
        self
    }

    /// Overrides default 2.5s display duration.
    pub fn duration(mut self, d: f32) -> Self {
        self.duration = d;
        self
    }

    /// Updates elapsed time by delta seconds `dt`. Automatically marks as dismissed when expired.
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            self.is_dismissed = true;
        }
    }

    /// Returns whether the toast is active and visible.
    #[inline]
    pub fn is_active(&self) -> bool {
        !self.is_dismissed && self.elapsed < self.duration
    }

    /// Returns the CoreAnimation spring interpolation fraction `[0.0, 1.0]`.
    pub fn entry_fraction(&self) -> f32 {
        self.spring.position_at(self.elapsed)
    }

    /// Explicitly marks the toast as dismissed.
    pub fn dismiss(&mut self) {
        self.is_dismissed = true;
    }
}

/// Manager container orchestrating active toast notifications.
#[derive(Default)]
pub struct MacToastManager {
    toasts: Vec<MacToast>,
}

impl MacToastManager {
    /// Creates an empty toast manager.
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    /// Adds a new toast to the manager queue.
    pub fn add(&mut self, toast: MacToast) {
        self.toasts.push(toast);
    }

    /// Shows an info message toast.
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(MacToast::new(message).level(ToastLevel::Info));
    }

    /// Shows a success message toast.
    pub fn success(&mut self, message: impl Into<String>) {
        self.add(MacToast::new(message).level(ToastLevel::Success));
    }

    /// Shows a warning message toast.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(MacToast::new(message).level(ToastLevel::Warning));
    }

    /// Shows an error message toast.
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(MacToast::new(message).level(ToastLevel::Error));
    }

    /// Updates all active toasts with time step `dt` and removes expired ones.
    pub fn update(&mut self, dt: f32) {
        for toast in &mut self.toasts {
            toast.update(dt);
        }
        self.toasts.retain(|t| t.is_active());
    }

    /// Number of active toasts.
    pub fn active_count(&self) -> usize {
        self.toasts.iter().filter(|t| t.is_active()).count()
    }

    /// Returns a slice of active toasts.
    pub fn active_toasts(&self) -> &[MacToast] {
        &self.toasts
    }

    /// Clears all active toasts.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Renders active toasts as floating pill banners in the given UI context.
    pub fn show(&mut self, ctx: &egui::Context, theme: &MacTheme) {
        if self.toasts.is_empty() {
            return;
        }
        let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.5);
        self.update(dt);

        if self.toasts.is_empty() {
            return;
        }
        let mut anim_in_flight = false;
        let mut next_wake = f32::MAX;
        for t in &self.toasts {
            if t.elapsed < t.spring.settling_duration() {
                anim_in_flight = true;
            } else {
                next_wake = next_wake.min((t.duration - t.elapsed).max(0.0));
            }
        }
        if anim_in_flight {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        } else if next_wake < f32::MAX {
            ctx.request_repaint_after_secs(next_wake);
        }

        let tm = theme.toast_metrics();
        let start_y = tm.top_margin.points();
        let toast_h = tm.height.points();
        let spacing = tm.spacing.points();
        let padding_h = tm.padding_x.points() * 0.5;
        let min_w = tm.min_width.points();
        let border_w = theme.popover_metrics().border_width.points();
        let font_id = FontId::proportional(tm.font_size.points());
        let on_accent = theme.on_accent_text();

        let fallback_w = theme.palette_metrics().width.points();
        let fallback_h = theme.palette_metrics().max_height.points();
        let fallback_screen = Vec2::new(fallback_w * 2.0, fallback_h * 2.0);
        let screen_rect = ctx
            .input(|i| i.raw.screen_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, fallback_screen));
        let area_id = Id::NULL.with("mac_toast_layer");

        Area::new(area_id)
            .order(Order::Tooltip)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                for (idx, toast) in self.toasts.iter_mut().enumerate() {
                    let fraction = toast.entry_fraction().clamp(0.0, 1.0 + 0.5);
                    let target_y = start_y + (idx as f32) * (toast_h + spacing);
                    let anim_y = -toast_h + (target_y - (-toast_h)) * fraction;

                    let galley = ui.painter().layout_no_wrap(
                        toast.message.clone(),
                        font_id.clone(),
                        theme.label(),
                    );
                    let toast_w = (galley.size().x + padding_h * 2.0).max(min_w);
                    let center_x = screen_rect.center().x;

                    let toast_rect = Rect::from_center_size(
                        Pos2::new(center_x, anim_y + toast_h * 0.5),
                        Vec2::new(toast_w, toast_h),
                    );

                    let painter = ui.painter();
                    let (bg, border) = match toast.level {
                        ToastLevel::Info => (theme.window_background(), theme.separator()),
                        ToastLevel::Success => (theme.system_green(), theme.system_green()),
                        ToastLevel::Warning => (theme.control_accent(), theme.control_accent()),
                        ToastLevel::Error => {
                            let (c_fill, c_border) = theme.traffic_light_close();
                            (c_fill, c_border)
                        }
                    };

                    painter.rect(
                        toast_rect,
                        CornerRadius::same((toast_h * 0.5) as u8),
                        bg,
                        Stroke::new(border_w, border),
                        StrokeKind::Inside,
                    );

                    let fg = match toast.level {
                        ToastLevel::Info => theme.label(),
                        _ => on_accent,
                    };

                    painter.galley(
                        Pos2::new(
                            toast_rect.center().x - galley.size().x * 0.5,
                            toast_rect.center().y - galley.size().y * 0.5,
                        ),
                        galley,
                        fg,
                    );

                    let resp = ui.interact(toast_rect, area_id.with(&toast.id), Sense::click());
                    if resp.clicked() {
                        toast.dismiss();
                    }
                }
            });
    }
}
