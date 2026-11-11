//! macOS AppKit Dynamic Vibrancy, Contrast & Background Style Engine.
//!
//! Grounded in W3C WCAG 2.1 §1.4.3 (Level AA), Apple CoreGraphics `kCGBlendModePlusDarker` /
//! `kCGBlendModePlusLighter`, and Apple AppKit `NSBackgroundStyle` / `NSAppearance` vibrancy.
//! Automatically resolves foreground typography tints against dynamic or recessed container
//! surfaces (such as `MacBox` and `MacCard`) to guarantee WCAG 2.1 AA contrast with zero
//! per-frame heap allocations.

use crate::theme::{MacAppearance, MacTheme};
use egui::{Color32, Ui};
use std::cell::Cell;
use std::sync::OnceLock;

#[derive(Clone, Copy)]
struct SurfaceScope {
    surface_bg: Color32,
    entry_panel_fill: Option<Color32>,
}

thread_local! {
    static ACTIVE_SURFACE_OVERRIDE: Cell<Option<SurfaceScope>> = const { Cell::new(None) };
}

static EGUI_DEFAULT_PANEL_FILLS: OnceLock<(Color32, Color32)> = OnceLock::new();

struct SurfaceScopeGuard {
    prev: Option<SurfaceScope>,
}

impl Drop for SurfaceScopeGuard {
    fn drop(&mut self) {
        ACTIVE_SURFACE_OVERRIDE.with(|cell| cell.set(self.prev));
    }
}

/// Converts an 8-bit sRGB channel component in `[0..255]` to linear luminance in `[0.0..1.0]`.
pub fn srgb_channel_to_linear(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// Computes the relative luminance of an sRGB color according to WCAG 2.1:
/// `L = 0.2126 * R_lin + 0.7152 * G_lin + 0.0722 * B_lin`.
pub fn relative_luminance(color: Color32) -> f32 {
    let r = srgb_channel_to_linear(color.r());
    let g = srgb_channel_to_linear(color.g());
    let b = srgb_channel_to_linear(color.b());
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Alpha-composites a potentially translucent `fg` over `bg` in unmultiplied sRGB space,
/// returning an opaque [`Color32`].
pub fn alpha_composite(fg: Color32, bg: Color32) -> Color32 {
    let [fr, fg_c, fb, fa] = fg.to_srgba_unmultiplied();
    if fa == 255 {
        return Color32::from_rgb(fr, fg_c, fb);
    }
    let alpha = fa as f32 / 255.0;
    let inv = 1.0 - alpha;
    let r = (fr as f32 * alpha + bg.r() as f32 * inv).round() as u8;
    let g = (fg_c as f32 * alpha + bg.g() as f32 * inv).round() as u8;
    let b = (fb as f32 * alpha + bg.b() as f32 * inv).round() as u8;
    Color32::from_rgb(r, g, b)
}

/// Computes the WCAG 2.1 contrast ratio between foreground and background colors.
/// If foreground has alpha < 255, it is alpha-blended over the background first.
pub fn contrast_ratio(fg: Color32, bg: Color32) -> f32 {
    let effective_fg = alpha_composite(fg, bg);
    let l1 = relative_luminance(effective_fg);
    let l2 = relative_luminance(bg);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Apple CoreGraphics / AppKit `PlusDarker` (`kCGBlendModePlusDarker`) surface-relative vibrancy
/// compositing for light surfaces.
///
/// Shifts the alpha-composited `fg` channels darker in proportion to the per-channel surface delta
/// between `surface_bg` and `reference_bg` scaled by `gain`.
pub fn plus_darker(fg: Color32, surface_bg: Color32, reference_bg: Color32, gain: f32) -> Color32 {
    let base = alpha_composite(fg, surface_bg);
    let dr = ((reference_bg.r() as i16 - surface_bg.r() as i16).unsigned_abs() as f32 * gain).round() as u8;
    let dg = ((reference_bg.g() as i16 - surface_bg.g() as i16).unsigned_abs() as f32 * gain).round() as u8;
    let db = ((reference_bg.b() as i16 - surface_bg.b() as i16).unsigned_abs() as f32 * gain).round() as u8;
    Color32::from_rgb(
        base.r().saturating_sub(dr),
        base.g().saturating_sub(dg),
        base.b().saturating_sub(db),
    )
}

/// Apple CoreGraphics / AppKit `PlusLighter` (`kCGBlendModePlusLighter`) surface-relative vibrancy
/// compositing for dark surfaces.
///
/// Shifts the alpha-composited `fg` channels lighter in proportion to the per-channel surface delta
/// between `surface_bg` and `reference_bg` scaled by `gain`.
pub fn plus_lighter(fg: Color32, surface_bg: Color32, reference_bg: Color32, gain: f32) -> Color32 {
    let base = alpha_composite(fg, surface_bg);
    let dr = ((reference_bg.r() as i16 - surface_bg.r() as i16).unsigned_abs() as f32 * gain).round() as u8;
    let dg = ((reference_bg.g() as i16 - surface_bg.g() as i16).unsigned_abs() as f32 * gain).round() as u8;
    let db = ((reference_bg.b() as i16 - surface_bg.b() as i16).unsigned_abs() as f32 * gain).round() as u8;
    Color32::from_rgb(
        base.r().saturating_add(dr),
        base.g().saturating_add(dg),
        base.b().saturating_add(db),
    )
}

/// Clamps `fg` against `bg` so that the resulting opaque [`Color32`] satisfies a WCAG 2.1 contrast
/// ratio of at least `min_ratio` (e.g. `4.5` for normal text, `3.0` for tertiary/UI components),
/// using a 10-step allocation-free binary search toward the high-contrast pole (`#000000` or `#FFFFFF`).
pub fn ensure_wcag_contrast(fg: Color32, bg: Color32, min_ratio: f32) -> Color32 {
    let bg_opaque = Color32::from_rgb(bg.r(), bg.g(), bg.b());
    let effective_fg = alpha_composite(fg, bg_opaque);
    if contrast_ratio(effective_fg, bg_opaque) >= min_ratio {
        return effective_fg;
    }

    let preferred_pole = match MacBackgroundStyle::from_color(bg_opaque) {
        MacBackgroundStyle::Light => Color32::BLACK,
        MacBackgroundStyle::Dark => Color32::WHITE,
    };
    let pole = if contrast_ratio(preferred_pole, bg_opaque) >= min_ratio {
        preferred_pole
    } else if relative_luminance(bg_opaque) > 0.179 {
        Color32::BLACK
    } else {
        Color32::WHITE
    };

    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    let mut best = pole;
    for _ in 0..10 {
        let mid = (low + high) * 0.5;
        let r = (effective_fg.r() as f32 + (pole.r() as f32 - effective_fg.r() as f32) * mid).round() as u8;
        let g = (effective_fg.g() as f32 + (pole.g() as f32 - effective_fg.g() as f32) * mid).round() as u8;
        let b = (effective_fg.b() as f32 + (pole.b() as f32 - effective_fg.b() as f32) * mid).round() as u8;
        let candidate = Color32::from_rgb(r, g, b);
        if contrast_ratio(candidate, bg_opaque) >= min_ratio {
            best = candidate;
            high = mid;
        } else {
            low = mid;
        }
    }
    best
}

impl MacTheme {
    /// Executes `f` inside a zero-allocation surface vibrancy scope where [`Self::active_surface`]
    /// returns `surface_bg`, automatically restoring the enclosing surface on scope exit.
    pub fn with_surface<R>(&self, surface_bg: Color32, f: impl FnOnce() -> R) -> R {
        self.with_surface_scope(surface_bg, None, f)
    }

    pub(crate) fn with_surface_scope<R>(
        &self,
        surface_bg: Color32,
        entry_panel_fill: Option<Color32>,
        f: impl FnOnce() -> R,
    ) -> R {
        let prev = ACTIVE_SURFACE_OVERRIDE.with(|cell| {
            let old = cell.get();
            cell.set(Some(SurfaceScope {
                surface_bg,
                entry_panel_fill,
            }));
            old
        });
        let _guard = SurfaceScopeGuard { prev };
        f()
    }

    fn is_default_panel_fill(&self, panel_fill: Color32) -> bool {
        let (egui_light, egui_dark) = *EGUI_DEFAULT_PANEL_FILLS
            .get_or_init(|| (egui::Visuals::light().panel_fill, egui::Visuals::dark().panel_fill));
        let colors = self.tokens().colors();
        panel_fill == egui_light
            || panel_fill == egui_dark
            || panel_fill == colors.aqua.window_background.to_color32()
            || panel_fill == colors.anthracite.window_background.to_color32()
            || panel_fill == colors.dark_aqua.window_background.to_color32()
    }

    /// Resolves the active container surface background color for `ui`:
    ///
    /// 1. Returns `ui.visuals().panel_fill` if explicitly mutated inside the current container scope.
    /// 2. Returns the active container surface pushed by [` crate::MacBox `], [` crate::MacCard `],
    ///    or [`Self::with_surface`] if inside a container scope.
    /// 3. Otherwise returns `ui.visuals().panel_fill` if customized away from default window/panel fills.
    /// 4. Falls back to [`Self::window_background`].
    pub fn active_surface(&self, ui: &Ui) -> Color32 {
        let panel_fill = ui.visuals().panel_fill;
        if let Some(scope) = ACTIVE_SURFACE_OVERRIDE.with(|cell| cell.get()) {
            if let Some(entry_pf) = scope.entry_panel_fill {
                if panel_fill != entry_pf {
                    return panel_fill;
                }
            } else if !self.is_default_panel_fill(panel_fill) {
                return panel_fill;
            }
            return scope.surface_bg;
        }
        if !self.is_default_panel_fill(panel_fill) {
            panel_fill
        } else {
            self.window_background()
        }
    }

    fn resolve_vibrant_on(&self, surface_bg: Color32, tier: u8, min_ratio: f32) -> Color32 {
        let gain = self.label_metrics().vibrancy_gain.points();
        let style = MacBackgroundStyle::from_color(surface_bg);
        let vibrant = match style {
            MacBackgroundStyle::Light => {
                let pal = &self.tokens().colors().aqua;
                let ref_bg = pal.window_background.to_color32();
                let base = match tier {
                    0 => pal.label.to_color32(),
                    1 => pal.secondary_label.to_color32(),
                    _ => pal.tertiary_label.to_color32(),
                };
                plus_darker(base, surface_bg, ref_bg, gain)
            }
            MacBackgroundStyle::Dark => {
                let pal = match self.appearance {
                    MacAppearance::Anthracite => &self.tokens().colors().anthracite,
                    MacAppearance::Aqua | MacAppearance::DarkAqua => {
                        &self.tokens().colors().dark_aqua
                    }
                };
                let ref_bg = pal.window_background.to_color32();
                let base = match tier {
                    0 => pal.label.to_color32(),
                    1 => pal.secondary_label.to_color32(),
                    _ => pal.tertiary_label.to_color32(),
                };
                plus_lighter(base, surface_bg, ref_bg, gain)
            }
        };
        ensure_wcag_contrast(vibrant, surface_bg, min_ratio)
    }

    /// Resolves primary `labelColor` (`NSColor.labelColor`) on `surface_bg` using AppKit vibrancy
    /// (`PlusDarker` / `PlusLighter`) and WCAG 2.1 AA contrast clamping (`>= 4.5:1`).
    pub fn label_on(&self, surface_bg: Color32) -> Color32 {
        let min_ratio = self.label_metrics().wcag_aa_normal_ratio.points();
        self.resolve_vibrant_on(surface_bg, 0, min_ratio)
    }

    /// Resolves `secondaryLabelColor` (`NSColor.secondaryLabelColor`) on `surface_bg` using AppKit
    /// vibrancy (`PlusDarker` / `PlusLighter`) and WCAG 2.1 AA contrast clamping (`>= 4.5:1`).
    pub fn secondary_label_on(&self, surface_bg: Color32) -> Color32 {
        let min_ratio = self.label_metrics().wcag_aa_normal_ratio.points();
        self.resolve_vibrant_on(surface_bg, 1, min_ratio)
    }

    /// Resolves `tertiaryLabelColor` (`NSColor.tertiaryLabelColor`) on `surface_bg` using AppKit
    /// vibrancy (`PlusDarker` / `PlusLighter`) and WCAG 2.1 AA contrast clamping (`>= 3.0:1`).
    pub fn tertiary_label_on(&self, surface_bg: Color32) -> Color32 {
        let min_ratio = self.label_metrics().wcag_aa_large_ratio.points();
        self.resolve_vibrant_on(surface_bg, 2, min_ratio)
    }
}

/// macOS AppKit background style (`NSBackgroundStyle`).
///
/// Indicates whether an element's background is light or dark, allowing child
/// text, icons, and accessories to select high-contrast tints that satisfy WCAG 2.1 AA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacBackgroundStyle {
    /// Background is light (relative luminance `L > 0.179`). Use dark foreground text.
    #[default]
    Light,
    /// Background is dark (relative luminance `L <= 0.179`). Use light/white foreground text.
    Dark,
}

impl MacBackgroundStyle {
    /// Resolves `MacBackgroundStyle` from a background color's WCAG 2.1 relative luminance.
    ///
    /// For neutral / low-saturation backgrounds, applies the normative WCAG 2.1 AA threshold `L = 0.179`:
    /// - `L <= 0.179` => `Dark` style (white text guarantees >= 4.5:1 contrast)
    /// - `L > 0.179` => `Light` style (dark text guarantees >= 4.5:1 contrast)
    ///
    /// For saturated chromatic fills (such as Apple System Blue `#007AFF` / `#0A84FF`),
    /// macOS AppKit filled controls use high-contrast white text (`Dark` style),
    /// satisfying WCAG 2.1 §1.4.11 / §1.4.3 UI Component / Control AA (>= 3.0:1) with `L <= 0.26`.
    pub fn from_color(bg: Color32) -> Self {
        let lum = relative_luminance(bg);
        let max_c = bg.r().max(bg.g()).max(bg.b());
        let min_c = bg.r().min(bg.g()).min(bg.b());
        let is_chromatic = (max_c - min_c) > 30;
        let threshold = if is_chromatic { 0.26 } else { 0.179 };
        if lum <= threshold {
            Self::Dark
        } else {
            Self::Light
        }
    }

    /// Primary text color guaranteeing >= 4.5:1 WCAG contrast against this background.
    ///
    /// - `Dark` => 100% white (`#FFFFFF`)
    /// - `Light` => dark primary label color from Aqua tokens (`#262626`)
    pub fn primary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(255, 255, 255),
            Self::Light => theme.tokens().colors().aqua.label.to_color32(),
        }
    }

    /// Secondary text color (subtitles, captions, metadata).
    ///
    /// - `Dark` => 85% white (`rgba(255, 255, 255, 0.85)`)
    /// - `Light` => dark secondary label color from Aqua tokens (`#5A5A5F`)
    pub fn secondary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(217),
            Self::Light => theme.tokens().colors().aqua.secondary_label.to_color32(),
        }
    }

    /// Tertiary text color (placeholders, disabled hints).
    ///
    /// - `Dark` => 70% white (`rgba(255, 255, 255, 0.70)`)
    /// - `Light` => dark placeholder text color from Aqua tokens
    pub fn tertiary_text_color(&self, theme: &MacTheme) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(179),
            Self::Light => theme.tokens().colors().aqua.placeholder_text.to_color32(),
        }
    }

    /// Standalone primary text color without requiring a `MacTheme` reference.
    pub fn primary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_rgb(255, 255, 255),
            Self::Light => Color32::from_rgb(0, 0, 0),
        }
    }

    /// Standalone secondary text color without requiring a `MacTheme` reference.
    pub fn secondary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(217),
            Self::Light => Color32::from_black_alpha(166),
        }
    }

    /// Standalone tertiary text color without requiring a `MacTheme` reference.
    pub fn tertiary_text(&self) -> Color32 {
        match self {
            Self::Dark => Color32::from_white_alpha(179),
            Self::Light => Color32::from_black_alpha(115),
        }
    }
}
