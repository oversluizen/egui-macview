//! Authentic macOS AppKit Image View (`NSImageView` / `NSImageCell` equivalent, REQ-013-02A).
//!
//! Supports vector [`MacSymbol`] glyphs, template icon monochrome surface tinting,
//! [`MacImageScaling`] modes, and [`MacImageFrameStyle`] borders.

use crate::layout::subpixel_snap_rect;
use crate::symbols::MacSymbol;
use crate::theme::MacTheme;
use egui::{CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

/// AppKit `NSImageScaling` scaling modes for [`MacImageView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacImageScaling {
    /// Scales the image down proportionally if it is larger than the destination rect (`NSImageScaleProportionallyDown`).
    #[default]
    ProportionallyDown,
    /// Stretches the image to fill the destination rect (`NSImageScaleAxesIndependently`).
    AxesIndependently,
    /// Draws the image at its natural size centered in the destination rect (`NSImageScaleNone`).
    None,
    /// Scales the image proportionally up or down to fit the destination rect (`NSImageScaleProportionallyUpOrDown`).
    ProportionallyUpOrDown,
}

impl MacImageScaling {
    /// Converts a raw NIB `NSScale` / `NSImageScaling` integer (`0..=3`) into [`MacImageScaling`].
    pub const fn from_nib_code(code: u8) -> Self {
        match code {
            1 => Self::AxesIndependently,
            2 => Self::None,
            3 => Self::ProportionallyUpOrDown,
            _ => Self::ProportionallyDown,
        }
    }
}

/// Computes the exact AppKit `-[NSImageCell imageRectForBounds:]` target rectangle for `bounds`,
/// logical point size `natural_pt`, [`MacImageScaling`], and optional `NSImageSymbolConfiguration` `symbol_point_size`.
pub fn compute_appkit_image_rect(
    bounds: Rect,
    natural_pt: Vec2,
    scaling: MacImageScaling,
    symbol_point_size: Option<f32>,
) -> Rect {
    let eff_nat = symbol_point_size.filter(|&pt| pt > 0.0).map(Vec2::splat).unwrap_or(natural_pt);
    if eff_nat.x <= 0.0 || eff_nat.y <= 0.0 || bounds.width() <= 0.0 || bounds.height() <= 0.0 {
        return bounds;
    }
    match scaling {
        MacImageScaling::ProportionallyDown => {
            let s = (bounds.width() / eff_nat.x).min(bounds.height() / eff_nat.y).min(1.0);
            Rect::from_center_size(bounds.center(), eff_nat * s)
        }
        MacImageScaling::AxesIndependently => bounds,
        MacImageScaling::None => Rect::from_center_size(bounds.center(), eff_nat),
        MacImageScaling::ProportionallyUpOrDown => {
            let s = (bounds.width() / eff_nat.x).min(bounds.height() / eff_nat.y);
            Rect::from_center_size(bounds.center(), eff_nat * s)
        }
    }
}

/// AppKit `NSImageFrameStyle` border styles for [`MacImageView`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacImageFrameStyle {
    /// No border frame (`NSImageFrameNone`).
    #[default]
    None,
    /// Thin photo border frame (`NSImageFramePhoto`).
    Photo,
    /// Rounded gray bezel border frame (`NSImageFrameGrayBezel`).
    GrayBezel,
}

/// Authentic macOS AppKit `NSImageView` control.
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacImageFrameStyle, MacImageScaling, MacImageView, MacSymbol, MacTheme};
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// MacImageView::from_symbol(MacSymbol::Gear)
///     .theme(&theme)
///     .template(true)
///     .scaling(MacImageScaling::ProportionallyDown)
///     .frame_style(MacImageFrameStyle::GrayBezel)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacImageView<'a> {
    symbol: Option<MacSymbol>,
    resource_name: Option<&'a str>,
    is_template: bool,
    tint_override: Option<egui::Color32>,
    scaling: MacImageScaling,
    symbol_point_size: Option<f32>,
    frame_style: MacImageFrameStyle,
    fixed_rect: Option<Rect>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacImageView<'a> {
    /// Creates a new image view displaying a vector [`MacSymbol`].
    pub fn from_symbol(symbol: MacSymbol) -> Self {
        Self {
            symbol: Some(symbol),
            resource_name: None,
            is_template: true,
            tint_override: None,
            scaling: MacImageScaling::ProportionallyDown,
            symbol_point_size: None,
            frame_style: MacImageFrameStyle::None,
            fixed_rect: None,
            theme: None,
        }
    }

    /// Creates a new image view referencing an AppKit `NSResourceName` icon identifier.
    pub fn from_resource_name(name: &'a str) -> Self {
        let is_template = name.ends_with("Template") || name.starts_with("NS") || !name.contains('/');
        let symbol = match name {
            "NSAddTemplate" | "plus" | "plus.circle" => Some(MacSymbol::Plus),
            "NSRemoveTemplate" | "minus" | "minus.circle" => Some(MacSymbol::Minus),
            "NSActionTemplate" | "gear" | "gearshape" | "ellipsis.circle" => Some(MacSymbol::Gear),
            "NSGoLeftTemplate" | "chevron.left" => Some(MacSymbol::ChevronLeft),
            "NSGoRightTemplate" | "chevron.right" => Some(MacSymbol::ChevronRight),
            "NSRefreshTemplate" | "arrow.clockwise" => Some(MacSymbol::ArrowClockwise),
            "NSStopProgressTemplate" | "xmark" | "xmark.circle" => Some(MacSymbol::Xmark),
            "NSSidebarTemplate" | "sidebar.left" | "sidebar.right" | "sidebar.leading" => Some(MacSymbol::SidebarLeading),
            "magnifyingglass" => Some(MacSymbol::Search),
            _ => None,
        };
        Self {
            symbol,
            resource_name: Some(name),
            is_template,
            tint_override: None,
            scaling: MacImageScaling::ProportionallyDown,
            symbol_point_size: None,
            frame_style: MacImageFrameStyle::None,
            fixed_rect: None,
            theme: None,
        }
    }

    /// Overrides the monochrome template foreground tint color (`REQ-013-10.6`).
    pub fn tint(mut self, color: egui::Color32) -> Self {
        self.tint_override = Some(color);
        self
    }

    /// Configures whether the image is treated as a monochrome `*Template` image tinted by the active surface.
    pub fn template(mut self, is_template: bool) -> Self {
        self.is_template = is_template;
        self
    }

    /// Configures the `NSImageScaling` mode.
    pub fn scaling(mut self, scaling: MacImageScaling) -> Self {
        self.scaling = scaling;
        self
    }

    /// Configures an explicit `NSImageSymbolConfiguration` point size override.
    pub fn symbol_point_size(mut self, point_size: Option<f32>) -> Self {
        self.symbol_point_size = point_size;
        self
    }

    /// Configures the `NSImageFrameStyle` border frame.
    pub fn frame_style(mut self, style: MacImageFrameStyle) -> Self {
        self.frame_style = style;
        self
    }

    /// Pins the image view to an exact NIB or scene layout rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Renders the image view inside `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let m = theme.image_view();
        let default_sz = m.default_size.points();

        let (rect, response) = if let Some(fixed) = self.fixed_rect {
            let r = subpixel_snap_rect(fixed, ui.ctx().pixels_per_point());
            let id = ui.next_auto_id().with("mac_image_view_fixed");
            let resp = ui.interact(r, id, Sense::hover());
            (r, resp)
        } else {
            ui.allocate_exact_size(Vec2::splat(default_sz), Sense::hover())
        };

        let surface_bg = theme.active_surface_at(ui, rect.center());
        let border_w = m.border_width.points();
        let bezel_r = m.bezel_corner_radius.points() as u8;
        let photo_pad = m.photo_frame_padding.points();

        let content_rect = match self.frame_style {
            MacImageFrameStyle::None => rect,
            MacImageFrameStyle::Photo => {
                ui.painter().rect(
                    rect,
                    CornerRadius::ZERO,
                    theme.control_background(),
                    Stroke::new(border_w, theme.box_border()),
                    StrokeKind::Inside,
                );
                rect.shrink(photo_pad)
            }
            MacImageFrameStyle::GrayBezel => {
                ui.painter().rect(
                    rect,
                    CornerRadius::same(bezel_r),
                    theme.box_background(),
                    Stroke::new(border_w, theme.box_border()),
                    StrokeKind::Inside,
                );
                rect.shrink(photo_pad)
            }
        };

        let res_name = self.resource_name.unwrap_or_default();
        let is_sec_sym = matches!(res_name, "tag" | "tag.fill" | "u.square" | "info.circle" | "info.circle.fill");
        let fg = self.tint_override.unwrap_or_else(|| {
            if is_sec_sym {
                theme.secondary_label_on(surface_bg)
            } else if self.is_template {
                theme.label_on(surface_bg)
            } else {
                theme.control_accent()
            }
        });

        let img_rect = compute_appkit_image_rect(
            content_rect,
            Vec2::splat(default_sz),
            self.scaling,
            self.symbol_point_size,
        );
        let min_dim = img_rect.width().min(img_rect.height());
        let glyph_sz = min_dim * 0.5;

        let c = img_rect.center();
        match res_name {
            "tag" | "tag.fill" if self.symbol.is_none() => {
                let s = (min_dim * 0.5 - photo_pad).max(photo_pad);
                let pts = vec![
                    Pos2::new(c.x, c.y - s),
                    Pos2::new(c.x + s, c.y - s),
                    Pos2::new(c.x + s, c.y),
                    Pos2::new(c.x, c.y + s),
                    Pos2::new(c.x - s, c.y),
                ];
                ui.painter().add(egui::Shape::closed_line(pts, Stroke::new(border_w, fg)));
                ui.painter().circle_filled(Pos2::new(c.x + s * 0.5, c.y - s * 0.5), border_w, fg);
            }
            "u.square" if self.symbol.is_none() => {
                let box_sz = (min_dim - bezel_r as f32).max(bezel_r as f32 * 2.0);
                let sq = Rect::from_center_size(c, Vec2::splat(box_sz));
                ui.painter().rect(
                    sq,
                    CornerRadius::same(photo_pad as u8),
                    egui::Color32::TRANSPARENT,
                    Stroke::new(border_w, fg),
                    StrokeKind::Inside,
                );
                let u_sz = (box_sz - bezel_r as f32).max(border_w);
                let galley = theme.layout_tracked_galley(ui, "U", FontId::proportional(u_sz), fg, box_sz);
                let pos = Pos2::new(c.x - galley.size().x * 0.5, c.y - galley.size().y * 0.5);
                ui.painter().galley(pos, galley, fg);
            }
            "info.circle" | "info.circle.fill" if self.symbol.is_none() => {
                let rad = (min_dim * 0.5 - border_w).max(photo_pad);
                let inner_fg = if res_name == "info.circle.fill" {
                    ui.painter().circle_filled(c, rad, fg);
                    theme.control_background()
                } else {
                    ui.painter().circle_stroke(c, rad, Stroke::new(border_w, fg));
                    fg
                };
                ui.painter().circle_filled(Pos2::new(c.x, c.y - rad * 0.5), border_w, inner_fg);
                ui.painter().line_segment(
                    [Pos2::new(c.x, c.y - border_w * 0.5), Pos2::new(c.x, c.y + rad * 0.5)],
                    Stroke::new(border_w + 0.5, inner_fg),
                );
            }
            _ => {
                let label_str = self.symbol.map_or(res_name, |sym| sym.as_str());
                if !label_str.is_empty() {
                    let galley =
                        theme.layout_tracked_galley(ui, label_str, FontId::proportional(glyph_sz.max(border_w)), fg, content_rect.width());
                    let pos = Pos2::new(c.x - galley.size().x * 0.5, c.y - galley.size().y * 0.5);
                    ui.painter().galley(pos, galley, fg);
                }
            }
        }

        response
    }
}
