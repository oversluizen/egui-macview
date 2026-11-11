//! Authentic macOS Disclosure Group & Disclosure Button (`SPEC-012`, `REQ-012-05` & `v0.6.5`).
//!
//! Grounded in Apple AppKit `NSBezelStyleDisclosure` and `NSBezelStyleRoundedDisclosure`
//! with `CASpring` micro-animations and W3C DTCG tokens via [`MacTheme`].

mod button;

pub use button::MacDisclosureButton;

use crate::motion::CASpring;
use crate::theme::MacTheme;
use egui::{
    Id, InnerResponse, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, UiBuilder, Vec2, WidgetText,
};
use std::hash::Hash;

/// Output returned by [`MacDisclosureGroup::show`], extending [`InnerResponse<Option<R>>`]
/// with trailing action button click detection (`trailing_clicked`).
#[derive(Debug, Clone)]
pub struct MacDisclosureResponse<R> {
    /// Output returned by the inner content closure when expanded (`Some(R)`), or `None` when collapsed.
    pub inner: Option<R>,
    /// Interaction response of the disclosure group header.
    pub response: Response,
    /// True if the optional trailing `+` action button was clicked on this frame.
    pub trailing_clicked: bool,
}

impl<R> From<MacDisclosureResponse<R>> for InnerResponse<Option<R>> {
    fn from(val: MacDisclosureResponse<R>) -> Self {
        InnerResponse::new(val.inner, val.response)
    }
}

/// Authentic macOS Disclosure Group (`NSBezelStyleDisclosure`).
///
/// Features animated triangular chevron ($0^\circ \to 90^\circ$) powered by `CASpring` physics,
/// Semibold section header typography with optical tracking, invariant $13 \times 13\,\text{pt}$
/// hit bounds, and an optional right-aligned `+` action button.
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
    trailing_add_button: bool,
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
            trailing_add_button: false,
        }
    }

    /// Injects dynamic theme to eliminate hardcoded metrics and colors.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Enables or disables a right-aligned `+` action button on the section header row.
    pub fn with_trailing_add_button(mut self, enabled: bool) -> Self {
        self.trailing_add_button = enabled;
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
    ) -> MacDisclosureResponse<R> {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.disclosure_group();

        let chevron_sz = metrics.chevron_size.points();
        let gap = metrics.label_gap.points();
        let header_font_sz = metrics.header_font_size.points();
        let trailing_btn_sz = metrics.trailing_button_size.points();
        let plus_half = metrics.trailing_plus_half_span.points();
        let plus_stroke_w = metrics.trailing_plus_stroke_width.points();
        let semibold_boost = theme.label_metrics().semibold_stroke_boost.points();

        let spring = CASpring::from_physical(
            metrics.spring_mass.points(),
            metrics.spring_stiffness.points(),
            metrics.spring_damping.points(),
        );
        let spring_key = self.id_salt.with("chevron_rot");

        let surface_bg = theme.active_surface(ui);
        let primary_fg = theme.label_on(surface_bg);
        let secondary_fg = theme.secondary_label_on(surface_bg);

        let (font_id, has_semibold_family) = theme.semibold_font_id(ui, header_font_sz);
        let galley = theme.layout_tracked_galley(
            ui,
            self.title.text(),
            font_id,
            primary_fg,
            f32::INFINITY,
        );

        let base_w = chevron_sz + gap + galley.size().x;
        let header_w = if self.trailing_add_button {
            ui.available_width().max(base_w + gap + trailing_btn_sz)
        } else {
            base_w
        };
        let header_h = chevron_sz
            .max(galley.size().y)
            .max(if self.trailing_add_button { trailing_btn_sz } else { 0.0 });

        let (header_rect, _) =
            ui.allocate_exact_size(Vec2::new(header_w, header_h), Sense::hover());

        let mut trailing_clicked = false;
        let toggle_rect = if self.trailing_add_button {
            let plus_rect = Rect::from_center_size(
                Pos2::new(header_rect.max.x - trailing_btn_sz * 0.5, header_rect.center().y),
                Vec2::splat(trailing_btn_sz),
            );
            let plus_resp = ui.interact(
                plus_rect,
                self.id_salt.with("trailing_add_btn"),
                Sense::click(),
            );
            if plus_resp.clicked() {
                trailing_clicked = true;
            }
            if ui.is_rect_visible(plus_rect) {
                let p = ui.painter_at(plus_rect);
                let c = plus_rect.center();
                let col = if plus_resp.hovered() {
                    primary_fg
                } else {
                    secondary_fg
                };
                let stroke = Stroke::new(plus_stroke_w, col);
                p.line_segment([Pos2::new(c.x - plus_half, c.y), Pos2::new(c.x + plus_half, c.y)], stroke);
                p.line_segment([Pos2::new(c.x, c.y - plus_half), Pos2::new(c.x, c.y + plus_half)], stroke);
            }
            Rect::from_min_max(
                header_rect.min,
                Pos2::new((plus_rect.min.x - gap).max(header_rect.min.x), header_rect.max.y),
            )
        } else {
            header_rect
        };

        let response = ui.interact(
            toggle_rect,
            self.id_salt.with("disclosure_hdr"),
            Sense::click(),
        );
        if response.clicked() && !trailing_clicked {
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
                primary_fg
            } else {
                secondary_fg
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
            painter.galley(label_pos, galley.clone(), primary_fg);
            if !has_semibold_family {
                painter.galley(
                    Pos2::new(label_pos.x + semibold_boost, label_pos.y),
                    galley,
                    primary_fg,
                );
            }
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

        MacDisclosureResponse {
            inner,
            response: response.with_new_rect(header_rect),
            trailing_clicked,
        }
    }
}
