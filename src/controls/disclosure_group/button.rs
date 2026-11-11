//! Authentic macOS Rounded Pill Disclosure Button (`NSBezelStyleRoundedDisclosure`).

use super::MacDisclosureGroup;
use crate::controls::MacControlSize;
use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{CornerRadius, Pos2, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS Rounded Pill Disclosure Button (`NSBezelStyleRoundedDisclosure`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacDisclosureButton, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut expanded = false;
///
/// ui.add(MacDisclosureButton::new(&mut expanded).theme(&theme));
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacDisclosureButton<'a> {
    is_expanded: &'a mut bool,
    size: MacControlSize,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacDisclosureButton<'a> {
    /// Creates a new rounded pill disclosure button.
    pub fn new(is_expanded: &'a mut bool) -> Self {
        Self {
            is_expanded,
            size: MacControlSize::Regular,
            theme: None,
        }
    }

    /// Sets the control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.size = size;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Returns the frame outer dimensions in points.
    pub fn frame_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.disclosure_group();
        match self.size {
            MacControlSize::Regular | MacControlSize::Large => {
                Vec2::new(m.button_regular_width.points(), m.button_regular_height.points())
            }
            MacControlSize::Small | MacControlSize::CompactInfobox => {
                Vec2::new(m.button_small_width.points(), m.button_small_height.points())
            }
            MacControlSize::Mini => {
                Vec2::new(m.button_mini_width.points(), m.button_mini_height.points())
            }
        }
    }

    /// Returns the intrinsic button dimensions in points.
    pub fn intrinsic_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let m = theme_cow.disclosure_group();
        match self.size {
            MacControlSize::Regular | MacControlSize::Large => {
                let sz = m.button_regular_intrinsic.points();
                Vec2::new(sz, sz)
            }
            MacControlSize::Small | MacControlSize::CompactInfobox => {
                let sz = m.button_small_intrinsic.points();
                Vec2::new(sz, sz)
            }
            MacControlSize::Mini => {
                let sz = m.button_mini_intrinsic.points();
                Vec2::new(sz, sz)
            }
        }
    }
}

impl<'a> Widget for MacDisclosureButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.disclosure_group();

        let desired_size = self.frame_size();
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *self.is_expanded = !*self.is_expanded;
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);
            let corner_r = rect.height() * 0.5;

            let fill = if response.is_pointer_button_down_on() {
                theme.selected_control()
            } else if response.hovered() {
                theme.control()
            } else {
                theme.control_background()
            };

            painter.rect(
                rect,
                CornerRadius::same(corner_r as u8),
                fill,
                Stroke::new(1.0, theme.separator()),
                StrokeKind::Inside,
            );

            let spring = CASpring::from_physical(
                metrics.spring_mass.points(),
                metrics.spring_stiffness.points(),
                metrics.spring_damping.points(),
            );
            let target_angle = MacDisclosureGroup::rotation_angle(*self.is_expanded);
            let anim_angle = if ui.is_rect_visible(rect) && crate::is_window_active(ui.ctx()) {
                spring.animate_f32(ui.ctx(), response.id.with("pill_rot"), target_angle)
            } else {
                target_angle
            };

            let center = rect.center();
            let chevron_sz = metrics.chevron_size.points() * 0.5;
            let cos = anim_angle.cos();
            let sin = anim_angle.sin();

            let rotate = |ox: f32, oy: f32| -> Pos2 {
                let rx = ox * cos - oy * sin;
                let ry = ox * sin + oy * cos;
                Pos2::new(center.x + rx, center.y + ry)
            };

            let p1 = rotate(-chevron_sz * 0.5, -chevron_sz);
            let p2 = rotate(-chevron_sz * 0.5, chevron_sz);
            let p3 = rotate(chevron_sz, 0.0);

            let glyph_color = if response.hovered() {
                theme.label_on(fill)
            } else {
                theme.secondary_label_on(fill)
            };
            painter.add(Shape::convex_polygon(
                vec![p1, p2, p3],
                glyph_color,
                Stroke::NONE,
            ));
        }

        response
    }
}
