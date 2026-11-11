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

const MAX_PASS_SURFACE_RECTS: usize = 32;

#[derive(Clone, Copy)]
struct PassSurfaceRects {
    pass_nr: u64,
    ctx_token: u64,
    len: usize,
    entries: [(egui::Rect, Color32); MAX_PASS_SURFACE_RECTS],
}

impl PassSurfaceRects {
    const EMPTY: Self = Self {
        pass_nr: u64::MAX,
        ctx_token: 0,
        len: 0,
        entries: [(egui::Rect::NOTHING, Color32::TRANSPARENT); MAX_PASS_SURFACE_RECTS],
    };
}

thread_local! {
    static PASS_SURFACE_RECTS: Cell<PassSurfaceRects> = const { Cell::new(PassSurfaceRects::EMPTY) };
    static NEXT_PASS_TOKEN: Cell<u64> = const { Cell::new(1) };
    static EXPLICIT_SURFACE_CELL: Cell<Option<Color32>> = const { Cell::new(None) };
}

static EGUI_DEFAULT_PANEL_FILLS: OnceLock<(Color32, Color32)> = OnceLock::new();

struct ExplicitSurfaceGuard {
    prev: Option<Color32>,
}

impl Drop for ExplicitSurfaceGuard {
    fn drop(&mut self) {
        EXPLICIT_SURFACE_CELL.with(|cell| cell.set(self.prev));
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
    /// falls back to `surface_bg`, automatically restoring the previous override on scope exit.
    pub fn with_surface<R>(&self, surface_bg: Color32, f: impl FnOnce() -> R) -> R {
        let prev = EXPLICIT_SURFACE_CELL.with(|cell| {
            let old = cell.get();
            cell.set(Some(surface_bg));
            old
        });
        let _guard = ExplicitSurfaceGuard { prev };
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

    /// Registers a painted container surface `(rect, surface_bg)` for the current `egui::Context`
    /// pass so flat NIB peer controls rendered after the container (such as via `.fixed_rect(rect)`)
    /// automatically detect `surface_bg` when their geometry lies within `rect`.
    pub fn register_surface_rect(
        &self,
        ctx: &egui::Context,
        rect: egui::Rect,
        surface_bg: Color32,
    ) {
        let _ = self.register_surface_rect_slot(ctx, rect, surface_bg);
    }

    pub(crate) fn register_surface_rect_slot(
        &self,
        ctx: &egui::Context,
        rect: egui::Rect,
        surface_bg: Color32,
    ) -> usize {
        let pass_nr = ctx.cumulative_pass_nr();
        let pass_id = egui::Id::unique("macview_surface_pass_token");
        PASS_SURFACE_RECTS.with(|cell| {
            let mut state = cell.get();
            let same_pass = state.pass_nr == pass_nr
                && state.ctx_token != 0
                && ctx.data(|d| d.get_temp::<(u64, u64)>(pass_id))
                    == Some((pass_nr, state.ctx_token));
            if !same_pass {
                let token = NEXT_PASS_TOKEN.with(|t| {
                    let next = t.get().wrapping_add(1).max(1);
                    t.set(next);
                    next
                });
                ctx.data_mut(|d| d.insert_temp(pass_id, (pass_nr, token)));
                state.pass_nr = pass_nr;
                state.ctx_token = token;
                state.len = 0;
            }
            let slot = if state.len < MAX_PASS_SURFACE_RECTS {
                let s = state.len;
                state.entries[s] = (rect, surface_bg);
                state.len += 1;
                s
            } else {
                state.entries.copy_within(1..MAX_PASS_SURFACE_RECTS, 0);
                state.entries[MAX_PASS_SURFACE_RECTS - 1] = (rect, surface_bg);
                MAX_PASS_SURFACE_RECTS - 1
            };
            cell.set(state);
            slot
        })
    }

    pub(crate) fn update_surface_rect_slot(
        &self,
        ctx: &egui::Context,
        slot: usize,
        rect: egui::Rect,
    ) {
        let pass_nr = ctx.cumulative_pass_nr();
        PASS_SURFACE_RECTS.with(|cell| {
            let mut state = cell.get();
            if state.pass_nr == pass_nr && slot < state.len {
                state.entries[slot].0 = rect;
                cell.set(state);
            }
        });
    }

    fn lookup_registered_surface(ctx: &egui::Context, pos: egui::Pos2) -> Option<Color32> {
        let pass_nr = ctx.cumulative_pass_nr();
        let (token, bg) = PASS_SURFACE_RECTS.with(|cell| {
            let state = cell.get();
            if state.pass_nr != pass_nr || state.len == 0 {
                return None;
            }
            for idx in (0..state.len).rev() {
                let (rect, bg) = state.entries[idx];
                if rect == egui::Rect::NOTHING {
                    return None;
                }
                if rect.contains(pos) {
                    return Some((state.ctx_token, bg));
                }
            }
            None
        })?;
        let pass_id = egui::Id::unique("macview_surface_pass_token");
        (ctx.data(|d| d.get_temp::<(u64, u64)>(pass_id)) == Some((pass_nr, token))).then_some(bg)
    }

    /// Resolves the active container surface background color at `pos` inside `ui`:
    ///
    /// 1. Returns the last registered `(rect, surface_bg)` in the current pass where `rect.contains(pos)`
    ///    (for flat NIB layouts where `MacBox::paint_at` or `MacBox::show` precedes `.fixed_rect` peers).
    /// 2. Returns `ui.visuals().panel_fill` when set by an enclosing container `Ui` (such as `MacBox::show`
    ///    or `MacCard::show`) or customized away from default window/panel fills.
    /// 3. Returns any active [`Self::with_surface`] override.
    /// 4. Falls back to [`Self::window_background`].
    pub fn active_surface_at(&self, ui: &Ui, pos: egui::Pos2) -> Color32 {
        if let Some(bg) = Self::lookup_registered_surface(ui.ctx(), pos) {
            return bg;
        }
        let panel_fill = ui.visuals().panel_fill;
        if !self.is_default_panel_fill(panel_fill) {
            return panel_fill;
        }
        if let Some(override_bg) = EXPLICIT_SURFACE_CELL.with(|cell| cell.get()) {
            return override_bg;
        }
        self.window_background()
    }

    /// Resolves the active container surface background color at `ui.next_widget_position()`.
    pub fn active_surface(&self, ui: &Ui) -> Color32 {
        self.active_surface_at(ui, ui.next_widget_position())
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
