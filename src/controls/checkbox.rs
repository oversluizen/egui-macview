//! Authentic macOS AppKit Square Checkbox (`NSButtonTypeSwitch`, `v0.6.5`).
//!
//! Grounded in macOS Sonoma 14.8.9 AppKit `NSButton(checkboxWithTitle:)` ground-truth metrics
//! and W3C DTCG tokens via [`MacTheme`].
//!
//! Unlike [`crate::MacToggle`] (`NSSwitch`, a `26x15pt` sliding pill), `MacCheckbox` renders
//! the native AppKit square rounded checkbox (`14x14pt` Regular, `12x12pt` Small, `10x10pt` Mini)
//! with a vector checkmark (or mixed `-` dash) and optical-tracked label aligned on the AppKit baseline.

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, FontId, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, Widget, WidgetText,
};

/// Authentic macOS AppKit Square Checkbox (`NSButtonTypeSwitch`).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacCheckbox, MacControlSize, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut checked = true;
/// MacCheckbox::new(&mut checked, "Show Grid Lines").control_size(MacControlSize::Regular).theme(&theme).show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacCheckbox<'a> {
    checked: &'a mut bool,
    label: WidgetText,
    mixed: bool,
    radio: bool,
    unaccented: bool,
    control_size: MacControlSize,
    fixed_rect: Option<Rect>,
    enabled: bool,
    theme: Option<&'a MacTheme>,
}

/// Authentic macOS AppKit Radio Button (`NSButtonTypeRadio`), backed by [`MacCheckbox`].
pub struct MacRadioButton<'a>(MacCheckbox<'a>);

impl<'a> MacRadioButton<'a> {
    /// Creates a new AppKit circular radio button (`NSButtonTypeRadio`) bound to `checked` with `label`.
    pub fn new(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self(MacCheckbox::new_radio(checked, label))
    }
    /// Creates a new AppKit circular radio button (`NSButtonTypeRadio`) bound to `checked` with `label`.
    pub fn new_radio(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self::new(checked, label)
    }
    /// Enables or disables circular radio button presentation.
    pub fn radio(mut self, radio: bool) -> Self { self.0 = self.0.radio(radio); self }
    /// Enables unaccented NIB / Graphite indicator presentation.
    pub fn unaccented(mut self, unaccented: bool) -> Self { self.0 = self.0.unaccented(unaccented); self }
    /// Sets the AppKit control size (`Regular` `14x14pt`, `Small` `12x12pt`, `Mini` `10x10pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self { self.0 = self.0.control_size(size); self }
    /// Pins the radio button to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self { self.0 = self.0.fixed_rect(rect); self }
    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self { self.0 = self.0.enabled(enabled); self }
    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self { self.0 = self.0.theme(theme); self }
    /// Resolves the indicator diameter (`14.0pt`, `12.0pt`, or `10.0pt`).
    pub fn box_size(&self) -> f32 { self.0.box_size() }
    /// Renders the radio button into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response { self.0.show(ui) }
}

impl<'a> Widget for MacRadioButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.0.ui(ui)
    }
}

impl<'a> MacCheckbox<'a> {
    /// Creates a new AppKit square checkbox bound to `checked` with `label`.
    pub fn new(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            label: label.into(),
            mixed: false,
            radio: false,
            unaccented: false,
            control_size: MacControlSize::Regular,
            fixed_rect: None,
            enabled: true,
            theme: None,
        }
    }

    /// Creates a new AppKit circular radio button (`NSButtonTypeRadio`) bound to `checked` with `label`.
    pub fn new_radio(checked: &'a mut bool, label: impl Into<WidgetText>) -> Self {
        Self::new(checked, label).radio(true)
    }

    /// Enables circular AppKit radio button presentation (`NSButtonTypeRadio`).
    pub fn radio(mut self, radio: bool) -> Self {
        self.radio = radio;
        self
    }

    /// Enables unaccented NIB / Graphite indicator presentation (white bezel with dark glyph).
    pub fn unaccented(mut self, unaccented: bool) -> Self {
        self.unaccented = unaccented;
        self
    }

    /// Sets the AppKit indeterminate / mixed state (`NSMixedState`, horizontal `-` dash).
    pub fn mixed(mut self, mixed: bool) -> Self {
        self.mixed = mixed;
        self
    }

    /// Sets the AppKit control size (`Regular` `14x14pt`, `Small` `12x12pt`, `Mini` `10x10pt`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Pins the checkbox to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Injects dynamic [`MacTheme`].
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Resolves the square indicator box dimension (`14.0pt`, `12.0pt`, or `10.0pt`).
    pub fn box_size(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        let cm = theme_cow.checkbox();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => cm.regular.box_size.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => cm.small.box_size.points(),
            MacControlSize::Mini => cm.mini.box_size.points(),
        }
    }

    /// Renders the checkbox into `ui`.
    pub fn show(self, ui: &mut Ui) -> Response {
        ui.add(self)
    }
}

impl<'a> Widget for MacCheckbox<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let cm = theme.checkbox();
        let lm = theme.label_metrics();

        let (sz_metrics, ascender, cell_h) = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => (&cm.regular, lm.regular_ascender.points(), lm.regular_height.points()),
            MacControlSize::Small | MacControlSize::CompactInfobox => (&cm.small, lm.small_ascender.points(), lm.small_height.points()),
            MacControlSize::Mini => (&cm.mini, lm.mini_ascender.points(), lm.mini_height.points()),
        };

        let box_sz = sz_metrics.box_size.points();
        let corner_r = sz_metrics.corner_radius.points();
        let gap = sz_metrics.label_gap.points();
        let font_sz = sz_metrics.font_size.points();
        let check_stroke_w = sz_metrics.checkmark_stroke.points();
        let baseline_from_top = sz_metrics.first_baseline_offset_from_top.points();
        let border_w = cm.border_width.points();

        let enabled = self.enabled && ui.is_enabled();
        let alpha = if enabled { 1.0 } else { theme.disabled_opacity() };
        let probe_pos = self.fixed_rect.map_or_else(
            || ui.next_widget_position(),
            |r| Pos2::new(r.left() + (box_sz * 0.5).min(r.width() * 0.5), r.center().y),
        );
        let surface_bg = theme.active_surface_at(ui, probe_pos);
        let text_color = if enabled { theme.label_on(surface_bg) } else { theme.tertiary_label_on(surface_bg) };
        let label_str = self.label.text();
        let galley = (!label_str.is_empty()).then(|| {
            theme.layout_tracked_galley(ui, label_str, FontId::proportional(font_sz), text_color, f32::INFINITY)
        });

        let desired_w = match (&self.fixed_rect, &galley) {
            (Some(r), _) => r.width(),
            (None, Some(g)) => box_sz + gap + g.size().x,
            (None, None) => box_sz,
        };
        let desired_h = self.fixed_rect.map(|r| r.height()).unwrap_or_else(|| box_sz.max(cell_h));

        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let snapped = subpixel_snap_rect(fixed, ppp);
            let id = ui.next_auto_id().with("mac_checkbox_fixed");
            ui.skip_ahead_auto_ids(1);
            (snapped, ui.interact(snapped, id, sense))
        } else {
            let (r, resp) = ui.allocate_exact_size(Vec2::new(desired_w, desired_h), sense);
            (subpixel_snap_rect(r, ppp), resp)
        };

        if enabled {
            let key_toggle = response.has_focus() && ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space));
            if response.clicked() || key_toggle {
                *self.checked = !*self.checked;
                response.mark_changed();
            }
        }

        let painter = ui.painter_at(rect);
        let box_rect = subpixel_snap_rect(
            Rect::from_min_size(Pos2::new(rect.left(), rect.center().y - box_sz * 0.5), Vec2::splat(box_sz)),
            ppp,
        );

        if enabled && response.has_focus() {
            theme.paint_focus_ring(&painter, box_rect, corner_r);
        }

        let is_active = *self.checked || self.mixed;
        let fill_color = if is_active && !self.unaccented {
            theme.control_accent().gamma_multiply(alpha)
        } else if response.is_pointer_button_down_on() && enabled {
            theme.control().gamma_multiply(alpha)
        } else {
            theme.control_background().gamma_multiply(alpha)
        };

        let stroke = if is_active && !self.unaccented {
            Stroke::NONE
        } else {
            Stroke::new(border_w, theme.separator().gamma_multiply(alpha))
        };
        let glyph_fg = if self.unaccented {
            theme.label().gamma_multiply(alpha)
        } else {
            theme.on_accent_text().gamma_multiply(alpha)
        };

        if self.radio {
            painter.circle(box_rect.center(), box_sz * 0.5, fill_color, stroke);
            if *self.checked {
                painter.circle_filled(box_rect.center(), box_sz * cm.mixed_bar_inset_ratio.points(), glyph_fg);
            }
        } else {
            painter.rect(box_rect, CornerRadius::same(corner_r.round() as u8), fill_color, stroke, StrokeKind::Inside);
            let glyph_stroke = Stroke::new(check_stroke_w, glyph_fg);
            if self.mixed {
                let (inset, cy) = (box_sz * cm.mixed_bar_inset_ratio.points(), box_rect.center().y);
                painter.line_segment([Pos2::new(box_rect.left() + inset, cy), Pos2::new(box_rect.right() - inset, cy)], glyph_stroke);
            } else if *self.checked {
                let p1 = Pos2::new(box_rect.left() + box_sz * cm.check_p1_x.points(), box_rect.top() + box_sz * cm.check_p1_y.points());
                let p2 = Pos2::new(box_rect.left() + box_sz * cm.check_p2_x.points(), box_rect.top() + box_sz * cm.check_p2_y.points());
                let p3 = Pos2::new(box_rect.left() + box_sz * cm.check_p3_x.points(), box_rect.top() + box_sz * cm.check_p3_y.points());
                painter.line_segment([p1, p2], glyph_stroke);
                painter.line_segment([p2, p3], glyph_stroke);
            }
        }

        if let Some(galley) = galley {
            let text_y = theme.baseline_text_y(rect, baseline_from_top, ascender, ppp);
            let text_pos = Pos2::new(box_rect.right() + gap, text_y);
            painter.galley(text_pos, galley, text_color);
        }

        response
    }
}
