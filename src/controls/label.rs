//! Authentic macOS AppKit Non-Editable Label (`NSTextField.labelWithString:`, `v0.6.5`).
//!
//! Grounded in macOS Sonoma 14.8.9 AppKit `NSTextField` label metrics and W3C DTCG tokens via [`MacTheme`].
//!
//! Provides exact AppKit typography sizing (`13pt` Regular, `11pt` Small, `9pt` Mini),
//! weight variants (`Regular`, `Medium`, `Semibold`, `Bold`), semantic label colors
//! (`Primary`, `Secondary`, `Tertiary`), optical tracking, and deterministic
//! cross-platform baseline positioning (`firstBaselineOffsetFromTop`).

use crate::controls::MacControlSize;
use crate::layout::{subpixel_snap, subpixel_snap_rect};
use crate::theme::MacTheme;
use egui::{Align, Color32, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget, WidgetText};

/// Font weight tiers for [`MacLabel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacFontWeight {
    /// Standard regular weight (`NSFontWeightRegular`).
    #[default]
    Regular,
    /// Medium weight (`NSFontWeightMedium`).
    Medium,
    /// Semibold weight (`NSFontWeightSemibold`, used for section headers).
    Semibold,
    /// Bold weight (`NSFontWeightBold`).
    Bold,
}

/// Semantic AppKit label color tiers for [`MacLabel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacLabelColor {
    /// Primary label color (`NSColor.labelColor`).
    #[default]
    Primary,
    /// Secondary label color (`NSColor.secondaryLabelColor`).
    Secondary,
    /// Tertiary label color (`NSColor.tertiaryLabelColor`).
    Tertiary,
}

/// Authentic macOS AppKit Non-Editable Label (`NSTextField.labelWithString:`).
///
/// # Example
/// ```rust
/// use egui_macview::{
///     MacAppearance, MacControlSize, MacFontWeight, MacLabel, MacLabelColor, MacTheme,
/// };
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
///
/// MacLabel::new("Output Resolution:")
///     .control_size(MacControlSize::Regular)
///     .weight(MacFontWeight::Semibold)
///     .color(MacLabelColor::Primary)
///     .theme(&theme)
///     .show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacLabel<'a> {
    text: WidgetText,
    control_size: MacControlSize,
    weight: MacFontWeight,
    color: MacLabelColor,
    alignment: Align,
    fixed_rect: Option<Rect>,
    theme: Option<&'a MacTheme>,
}

impl<'a> MacLabel<'a> {
    /// Creates a new AppKit non-editable label with `text`.
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            control_size: MacControlSize::Regular,
            weight: MacFontWeight::Regular,
            color: MacLabelColor::Primary,
            alignment: Align::Min,
            fixed_rect: None,
            theme: None,
        }
    }

    /// Sets the AppKit control size (`Regular` `13pt`, `Small` `11pt`, `Mini` `9pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Sets the font weight (`Regular`, `Medium`, `Semibold`, `Bold`).
    pub fn weight(mut self, weight: MacFontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Sets the semantic label color (`Primary`, `Secondary`, `Tertiary`).
    pub fn color(mut self, color: MacLabelColor) -> Self {
        self.color = color;
        self
    }

    /// Sets horizontal text alignment (`Align::Min`, `Align::Center`, `Align::Max`).
    pub fn alignment(mut self, alignment: Align) -> Self {
        self.alignment = alignment;
        self
    }

    /// Pins the label to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Resolves the font size in points (`13.0pt`, `11.0pt`, or `9.0pt`).
    pub fn font_size(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let lm = theme_cow.label_metrics();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => lm.regular_font_size.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => lm.small_font_size.points(),
            MacControlSize::Mini => lm.mini_font_size.points(),
        }
    }

    /// Resolves the semantic [`Color32`] from `theme`.
    pub fn resolved_color(&self, theme: &MacTheme) -> Color32 {
        match self.color {
            MacLabelColor::Primary => theme.label(),
            MacLabelColor::Secondary => theme.secondary_label(),
            MacLabelColor::Tertiary => theme.tertiary_label(),
        }
    }

    /// Renders the label into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacLabel<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let lm = theme.label_metrics();

        let (font_sz, cell_h, baseline_from_top, ascender) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (
                lm.regular_font_size.points(),
                lm.regular_height.points(),
                lm.regular_baseline_from_top.points(),
                lm.regular_ascender.points(),
            ),
            MacControlSize::Small | MacControlSize::CompactInfobox => (
                lm.small_font_size.points(),
                lm.small_height.points(),
                lm.small_baseline_from_top.points(),
                lm.small_ascender.points(),
            ),
            MacControlSize::Mini => (
                lm.mini_font_size.points(),
                lm.mini_height.points(),
                lm.mini_baseline_from_top.points(),
                lm.mini_ascender.points(),
            ),
        };

        let text_color = self.resolved_color(theme);
        let is_heavy = matches!(
            self.weight,
            MacFontWeight::Medium | MacFontWeight::Semibold | MacFontWeight::Bold
        );
        let (font_id, has_semibold_family) = if is_heavy {
            theme.semibold_font_id(ui, font_sz)
        } else {
            (FontId::proportional(font_sz), false)
        };

        let wrap_w = self.fixed_rect.map_or(f32::INFINITY, |r| r.width());
        let galley =
            theme.layout_tracked_galley(ui, self.text.text(), font_id, text_color, wrap_w);

        let ppp = ui.ctx().pixels_per_point();
        let (rect, response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_label_fixed");
            let resp = ui.interact(snapped, id, Sense::hover());
            (snapped, resp)
        } else {
            let desired = Vec2::new(galley.size().x, cell_h.max(galley.size().y));
            let (r, resp) = ui.allocate_exact_size(desired, Sense::hover());
            (subpixel_snap_rect(r, ppp), resp)
        };

        let painter = ui.painter_at(rect);
        let text_y = subpixel_snap(rect.top() + (baseline_from_top - ascender), ppp);
        let text_x = match self.alignment {
            Align::Min => rect.left(),
            Align::Center => rect.center().x - galley.size().x * 0.5,
            Align::Max => (rect.right() - galley.size().x).max(rect.left()),
        };
        let pos = Pos2::new(text_x, text_y);
        painter.galley(pos, galley.clone(), text_color);

        if is_heavy && !has_semibold_family {
            let boost = match self.weight {
                MacFontWeight::Bold => lm.bold_stroke_boost.points(),
                MacFontWeight::Medium | MacFontWeight::Semibold => lm.semibold_stroke_boost.points(),
                MacFontWeight::Regular => 0.0,
            };
            if boost > 0.0 {
                painter.galley(Pos2::new(pos.x + boost, pos.y), galley, text_color);
            }
        }

        response
    }
}
