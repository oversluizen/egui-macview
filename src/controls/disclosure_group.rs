//! Authentic macOS Disclosure Group & Disclosure Button (SPEC-012, REQ-012-05).
//!
//! Grounded in Apple AppKit `NSBezelStyleDisclosure` and `NSBezelStyleRoundedDisclosure`
//! with `CASpring` micro-animations and W3C DTCG tokens via [`MacTheme`].
//!
//! # Engineering Rationale & Apple HIG Measurements
//!
//! On macOS Sonoma 14.8.9, disclosure triangles adhere to specific HIG standards:
//!
//! - **Invariant 13.0 x 13.0 pt Chevron (`chevron_size`)**:
//!   In `NSBezelStyleDisclosure`, the chevron container is strictly invariant at $13.0 \times 13.0\,\text{pt}$
//!   regardless of whether surrounding text is Regular, Small, or Large. This guarantees identical
//!   vertical optical centering and indentation across inspector panes, sidebar trees, and forms.
//! - **6.0 pt Horizontal Label Gap (`label_gap`)**:
//!   The gap between the chevron's trailing edge and the title label is invariant at $6.0\,\text{pt}$,
//!   matching AppKit's standard token for disclosure controls.
//! - **CoreAnimation Spring Physics (`CASpring(1.0, 300.0, 25.0)`)**:
//!   The $0^\circ \to 90^\circ$ chevron rotation is driven by a natural physical spring with
//!   mass $m = 1.0$, stiffness $k = 300.0\,\text{N/m}$, and damping $c = 25.0\,\text{N}\cdot\text{s/m}$.
//!   This produces Apple's snappy, slightly underdamped micro-interaction with ~180ms settling duration.

use crate::controls::MacControlSize;
use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Id, InnerResponse, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui,
    UiBuilder, Vec2, Widget, WidgetText,
};
use std::hash::Hash;

/// Authentic macOS Disclosure Group (`NSBezelStyleDisclosure`).
///
/// Features animated triangular chevron ($0^\circ \to 90^\circ$) powered by `CASpring` physics,
/// invariant $13 \times 13\,\text{pt}$ hit bounds, and accessible keyboard toggle.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacDisclosureGroup, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut expanded = true;
///
/// MacDisclosureGroup::new("export_details", "Export Details", &mut expanded)
///     .theme(&theme)
///     .show(ui, |ui| {
///         ui.label("Color Profile: Display P3");
///     });
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacDisclosureGroup<'a> {
    id_salt: Id,
    title: WidgetText,
    is_expanded: &'a mut bool,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacDisclosureGroup<'a> {
    /// Creates a new disclosure group bound to an expansion boolean state.
    pub fn new(
        id_salt: impl Hash + std::fmt::Debug,
        title: impl Into<WidgetText>,
        is_expanded: &'a mut bool,
    ) -> Self {
        Self {
            id_salt: Id::NULL.with(id_salt),
            title: title.into(),
            is_expanded,
            theme: None,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Invariant chevron bounding box size ($13.0 \times 13.0\,\text{pt}$).
    pub fn chevron_size(&self) -> Vec2 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let metrics = theme_cow.disclosure_group();
        let sz = metrics.chevron_size.points();
        Vec2::new(sz, sz)
    }

    /// Horizontal gap between chevron and trailing label ($6.0\,\text{pt}$).
    pub fn label_gap(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.disclosure_group().label_gap.points()
    }

    /// Closed-form rotation angle for collapsed ($0^\circ$) vs expanded ($90^\circ$).
    pub fn rotation_angle(expanded: bool) -> f32 {
        if expanded {
            std::f32::consts::FRAC_PI_2
        } else {
            0.0
        }
    }

    /// Shows the disclosure group header and nested contents if expanded.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.disclosure_group();

        let chevron_sz = metrics.chevron_size.points();
        let gap = metrics.label_gap.points();
        let spring = CASpring::from_physical(
            metrics.spring_mass.points(),
            metrics.spring_stiffness.points(),
            metrics.spring_damping.points(),
        );

        let spring_key = self.id_salt.with("chevron_rot");

        let galley = self.title.into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Button,
        );
        let header_w = chevron_sz + gap + galley.size().x;
        let header_h = chevron_sz.max(galley.size().y);
        let header_desired = Vec2::new(header_w, header_h);

        let (header_rect, response) = ui.allocate_exact_size(header_desired, Sense::click());
        if response.clicked() {
            *self.is_expanded = !*self.is_expanded;
        }

        let target_angle = Self::rotation_angle(*self.is_expanded);

        let anim_angle = if ui.is_rect_visible(header_rect) && crate::is_window_active(ui.ctx()) {
            spring.animate_f32(ui.ctx(), spring_key, target_angle)
        } else {
            target_angle
        };

        if ui.is_rect_visible(header_rect) {
            let painter = ui.painter_at(header_rect);
            let chevron_center = Pos2::new(
                header_rect.min.x + chevron_sz * 0.5,
                header_rect.min.y + header_h * 0.5,
            );

            // Draw animated rotating triangle chevron
            let cos = anim_angle.cos();
            let sin = anim_angle.sin();
            let half_w = chevron_sz * 0.5;
            let half_h = chevron_sz * 0.5;

            let rotate = |ox: f32, oy: f32| -> Pos2 {
                let rx = ox * cos - oy * sin;
                let ry = ox * sin + oy * cos;
                Pos2::new(chevron_center.x + rx, chevron_center.y + ry)
            };

            let p1 = rotate(-half_w * 0.5, -half_h);
            let p2 = rotate(-half_w * 0.5, half_h);
            let p3 = rotate(half_w, 0.0);

            let glyph_color = if response.hovered() {
                theme.label()
            } else {
                theme.secondary_label()
            };
            painter.add(Shape::convex_polygon(
                vec![p1, p2, p3],
                glyph_color,
                Stroke::NONE,
            ));

            let label_pos = Pos2::new(
                header_rect.min.x + chevron_sz + gap,
                header_rect.min.y + (header_h - galley.size().y) * 0.5,
            );
            painter.galley(label_pos, galley, theme.label());
        }

        let inner = if *self.is_expanded {
            let indent = chevron_sz + gap;
            let content_rect = Rect::from_min_max(
                Pos2::new(ui.min_rect().min.x + indent, ui.cursor().min.y),
                Pos2::new(ui.max_rect().max.x, ui.max_rect().max.y),
            );
            let mut child_ui = ui.new_child(
                UiBuilder::new()
                    .id_salt(self.id_salt.with("content"))
                    .max_rect(content_rect),
            );
            let result = add_contents(&mut child_ui);
            ui.advance_cursor_after_rect(child_ui.min_rect());
            Some(result)
        } else {
            None
        };

        InnerResponse::new(inner, response)
    }
}

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

            // Draw center rotating chevron
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
                theme.label()
            } else {
                theme.secondary_label()
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
