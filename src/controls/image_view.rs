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
    scaling: MacImageScaling,
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
            scaling: MacImageScaling::ProportionallyDown,
            frame_style: MacImageFrameStyle::None,
            fixed_rect: None,
            theme: None,
        }
    }

    /// Creates a new image view referencing an AppKit `NSResourceName` icon identifier.
    pub fn from_resource_name(name: &'a str) -> Self {
        let is_template = name.ends_with("Template") || name.starts_with("NS");
        let symbol = match name {
            "NSAddTemplate" => Some(MacSymbol::Plus),
            "NSRemoveTemplate" => Some(MacSymbol::Minus),
            "NSActionTemplate" => Some(MacSymbol::Gear),
            "NSGoLeftTemplate" => Some(MacSymbol::ChevronLeft),
            "NSGoRightTemplate" => Some(MacSymbol::ChevronRight),
            "NSRefreshTemplate" => Some(MacSymbol::ArrowClockwise),
            "NSStopProgressTemplate" => Some(MacSymbol::Xmark),
            _ => None,
        };
        Self {
            symbol,
            resource_name: Some(name),
            is_template,
            scaling: MacImageScaling::ProportionallyDown,
            frame_style: MacImageFrameStyle::None,
            fixed_rect: None,
            theme: None,
        }
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

        let fg = if self.is_template { theme.label_on(surface_bg) } else { theme.control_accent() };

        let min_dim = content_rect.width().min(content_rect.height());
        let glyph_sz = match self.scaling {
            MacImageScaling::None => default_sz.min(min_dim),
            MacImageScaling::ProportionallyDown => default_sz.min(min_dim * 0.5),
            MacImageScaling::AxesIndependently | MacImageScaling::ProportionallyUpOrDown => min_dim * 0.5,
        };

        let label_str = if let Some(sym) = self.symbol { sym.as_str() } else { self.resource_name.unwrap_or_default() };

        if !label_str.is_empty() {
            let galley = theme.layout_tracked_galley(ui, label_str, FontId::proportional(glyph_sz.max(border_w)), fg, content_rect.width());
            let pos = Pos2::new(content_rect.center().x - galley.size().x * 0.5, content_rect.center().y - galley.size().y * 0.5);
            ui.painter().galley(pos, galley, fg);
        }

        response
    }
}
