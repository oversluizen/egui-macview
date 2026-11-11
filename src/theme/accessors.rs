//! Metric, Typography, Motion, and Contrast Accessors on [`MacTheme`].

use super::{MacBackgroundStyle, MacTheme};
use crate::tokens::{
    BoxViewMetrics, ButtonMetrics, CheckboxMetrics, CollectionViewMetrics, ColorWellMetrics,
    ComboBoxMetrics, ContextMenuMetrics, DatePickerMetrics, DisclosureGroupMetrics,
    FocusRingMetrics, FooterMetrics, HelpButtonMetrics, InlineActionButtonMetrics, LabelMetrics,
    LayoutMetrics, LevelIndicatorMetrics, MacControlMetrics, MacMotionTokens, MacTypographyTokens,
    MenuBarMetrics, NavigationMetrics, OutlineViewMetrics, PaletteMetrics, PathControlMetrics,
    PlayerViewMetrics, PopUpButtonMetrics, PopoverMetrics, ProgressIndicatorMetrics,
    RuleEditorMetrics, ScrollViewMetrics, SearchFieldMetrics, SegmentedMetrics, SliderMetrics,
    SplitViewMetrics, StackViewMetrics, StepperMetrics, TabViewMetrics, TableViewMetrics,
    TextFieldMetrics, ThumbnailCellMetrics, ToastMetrics, ToggleMetrics, TokenFieldMetrics,
    ToolbarMetrics, TrimScrubberMetrics,
};
use egui::{Color32, FontId, Galley, TextFormat, Ui};
use std::sync::Arc;

const fn tracking_table_key(pt: i32) -> Option<&'static str> {
    match pt {
        6 => Some("pt_6"),
        7 => Some("pt_7"),
        8 => Some("pt_8"),
        9 => Some("pt_9"),
        10 => Some("pt_10"),
        11 => Some("pt_11"),
        12 => Some("pt_12"),
        13 => Some("pt_13"),
        14 => Some("pt_14"),
        15 => Some("pt_15"),
        16 => Some("pt_16"),
        17 => Some("pt_17"),
        18 => Some("pt_18"),
        19 => Some("pt_19"),
        20 => Some("pt_20"),
        21 => Some("pt_21"),
        22 => Some("pt_22"),
        23 => Some("pt_23"),
        24 => Some("pt_24"),
        _ => None,
    }
}

impl MacTheme {
    /// Resolves Apple HIG / SF Pro optical tracking (`extra_letter_spacing` in points)
    /// for `size_pt` from `self.typography().tracking_table`, falling back to
    /// [`crate::tokens::sf_pro_tracking`] for fractional or out-of-range font sizes.
    pub fn tracking_for_font_size(&self, size_pt: f32) -> f32 {
        let rounded = size_pt.round();
        if (size_pt - rounded).abs() == 0.0 {
            if let Some(key) = tracking_table_key(rounded as i32) {
                if let Some(entry) = self.typography().tracking_table.get(key) {
                    return entry.tracking_pt;
                }
            }
        }
        crate::tokens::sf_pro_tracking(size_pt)
    }

    /// Resolves the `Inter-SemiBold` [`FontId`] when registered in `ui.ctx()`, or falls back
    /// to [`FontId::proportional`], returning `(font_id, has_semibold_family)` with zero
    /// per-widget `Arc<str>` or `Vec<FontFamily>` heap allocations.
    pub fn semibold_font_id(&self, ui: &Ui, size_pt: f32) -> (FontId, bool) {
        static INTER_SEMIBOLD_FAMILY: std::sync::OnceLock<egui::FontFamily> =
            std::sync::OnceLock::new();
        let family = INTER_SEMIBOLD_FAMILY.get_or_init(|| {
            egui::FontFamily::Name(crate::symbols::INTER_SEMIBOLD_FONT_NAME.into())
        });
        let cache_id = egui::Id::unique("macview_has_inter_semibold");
        let pass_nr = ui.ctx().cumulative_pass_nr();
        let cached = ui.ctx().data(|d| d.get_temp::<(u64, bool)>(cache_id));
        let has_semibold = match cached {
            Some((_, true)) => true,
            Some((cached_pass, false)) if cached_pass == pass_nr => false,
            _ => {
                let found = ui.fonts(|f| f.families().contains(family));
                ui.ctx().data_mut(|d| d.insert_temp(cache_id, (pass_nr, found)));
                found
            }
        };
        if has_semibold {
            (FontId::new(size_pt, family.clone()), true)
        } else {
            (FontId::proportional(size_pt), false)
        }
    }

    /// Constructs an [`egui::TextFormat`] populated with optical letter spacing (`extra_letter_spacing`)
    /// from the Apple HIG tracking table for `font_id.size`.
    pub fn tracked_text_format(&self, font_id: FontId, color: Color32) -> TextFormat {
        TextFormat {
            font_id: font_id.clone(),
            color,
            extra_letter_spacing: self.tracking_for_font_size(font_id.size),
            ..Default::default()
        }
    }

    /// Lays out a text [`Galley`] with Apple HIG optical tracking (`extra_letter_spacing`)
    /// automatically applied from `self.typography().tracking_table`.
    pub fn layout_tracked_galley(
        &self,
        ui: &Ui,
        text: &str,
        font_id: FontId,
        color: Color32,
        wrap_width: f32,
    ) -> Arc<Galley> {
        let mut job = egui::text::LayoutJob::single_section(
            text.to_owned(),
            self.tracked_text_format(font_id, color),
        );
        job.wrap.max_width = wrap_width;
        ui.painter().layout_job(job)
    }

    /// Returns control metrics tokens.
    pub fn control_metrics(&self) -> &MacControlMetrics { self.tokens.control_metrics() }
    /// Returns button size and geometry tokens.
    pub fn button(&self) -> &ButtonMetrics { &self.tokens.control_metrics().button }
    /// Returns text field and input cell size/geometry tokens.
    pub fn text_field(&self) -> &TextFieldMetrics { &self.tokens.control_metrics().text_field }
    /// Returns segmented control geometry tokens.
    pub fn segmented(&self) -> &SegmentedMetrics { &self.tokens.control_metrics().segmented }
    /// Returns focus ring halo tokens.
    pub fn focus_ring(&self) -> &FocusRingMetrics { &self.tokens.control_metrics().focus_ring }
    /// Returns toolbar layout and hit-target tokens.
    pub fn toolbar_metrics(&self) -> &ToolbarMetrics { self.tokens.toolbar_metrics() }
    /// Returns popover and floating panel geometry tokens.
    pub fn popover_metrics(&self) -> &PopoverMetrics { self.tokens.popover_metrics() }
    /// Returns split view divider and pane geometry tokens.
    pub fn split_view(&self) -> &SplitViewMetrics { self.tokens.split_view_metrics() }
    /// Returns layout container metrics (`scroll_view`, `stack_view`).
    pub fn layout_metrics(&self) -> &LayoutMetrics { self.tokens.layout_metrics() }
    /// Returns scroll view overlay scrollbar geometry tokens.
    pub fn scroll_view(&self) -> &ScrollViewMetrics { self.tokens.scroll_view_metrics() }
    /// Returns stack view spacing, padding, and header tokens.
    pub fn stack_view(&self) -> &StackViewMetrics { self.tokens.stack_view_metrics() }
    /// Returns outline view row, indentation, and chevron tokens.
    pub fn outline_view(&self) -> &OutlineViewMetrics { self.tokens.outline_view_metrics() }
    /// Returns slider track, thumb, and tick tokens.
    pub fn slider(&self) -> &SliderMetrics { self.tokens.slider_metrics() }
    /// Returns thumbnail cell preview tokens.
    pub fn thumbnail(&self) -> &ThumbnailCellMetrics { self.tokens.thumbnail_metrics() }
    /// Returns pop-up button bezel and menu tokens.
    pub fn popup_button(&self) -> &PopUpButtonMetrics { self.tokens.popup_button_metrics() }
    /// Returns search field pill, icon, and clear button tokens.
    pub fn search_field(&self) -> &SearchFieldMetrics { self.tokens.search_field_metrics() }
    /// Returns numeric stepper control tokens.
    pub fn stepper(&self) -> &StepperMetrics { self.tokens.stepper_metrics() }
    /// Returns color well swatch and picker tokens.
    pub fn color_well(&self) -> &ColorWellMetrics { self.tokens.color_well_metrics() }
    /// Returns toggle switch (`NSSwitch`) tokens.
    pub fn toggle(&self) -> &ToggleMetrics { self.tokens.toggle_metrics() }
    /// Returns navigation and feedback metrics tokens.
    pub fn navigation_metrics(&self) -> &NavigationMetrics { self.tokens.navigation_metrics() }
    /// Returns menu bar and dropdown menu tokens.
    pub fn menu_bar_metrics(&self) -> &MenuBarMetrics { self.tokens.menu_bar_metrics() }
    /// Returns canvas footer bar tokens.
    pub fn footer_metrics(&self) -> &FooterMetrics { self.tokens.footer_metrics() }
    /// Returns Spotlight command palette tokens.
    pub fn palette_metrics(&self) -> &PaletteMetrics { self.tokens.palette_metrics() }
    /// Returns context menu popup tokens.
    pub fn context_menu_metrics(&self) -> &ContextMenuMetrics { self.tokens.context_menu_metrics() }
    /// Returns floating toast notification tokens.
    pub fn toast_metrics(&self) -> &ToastMetrics { self.tokens.toast_metrics() }
    /// Returns progress indicator bar and spinning fin tokens.
    pub fn progress_indicator(&self) -> &ProgressIndicatorMetrics { self.tokens.progress_indicator_metrics() }
    /// Returns table view and sortable header tokens.
    pub fn table_view(&self) -> &TableViewMetrics { self.tokens.table_view_metrics() }
    /// Returns token field pill capsule tokens.
    pub fn token_field(&self) -> &TokenFieldMetrics { self.tokens.token_field_metrics() }
    /// Returns level indicator VU meter tokens.
    pub fn level_indicator(&self) -> &LevelIndicatorMetrics { self.tokens.level_indicator_metrics() }
    /// Returns AVKit trim scrubber timeline tokens.
    pub fn trim_scrubber(&self) -> &TrimScrubberMetrics { self.tokens.trim_scrubber_metrics() }
    /// Returns path control breadcrumb bar tokens.
    pub fn path_control(&self) -> &PathControlMetrics { self.tokens.path_control_metrics() }
    /// Returns disclosure group & button tokens.
    pub fn disclosure_group(&self) -> &DisclosureGroupMetrics { self.tokens.disclosure_group_metrics() }
    /// Returns combo box tokens.
    pub fn combo_box(&self) -> &ComboBoxMetrics { self.tokens.combo_box_metrics() }
    /// Returns tab view tokens.
    pub fn tab_view(&self) -> &TabViewMetrics { self.tokens.tab_view_metrics() }
    /// Returns date picker tokens.
    pub fn date_picker(&self) -> &DatePickerMetrics { self.tokens.date_picker_metrics() }
    /// Returns collection view tokens.
    pub fn collection_view(&self) -> &CollectionViewMetrics { self.tokens.collection_view_metrics() }
    /// Returns player view tokens.
    pub fn player_view(&self) -> &PlayerViewMetrics { self.tokens.player_view_metrics() }
    /// Returns box view tokens.
    pub fn box_view(&self) -> &BoxViewMetrics { self.tokens.box_view_metrics() }
    /// Returns rule editor tokens.
    pub fn rule_editor(&self) -> &RuleEditorMetrics { self.tokens.rule_editor_metrics() }
    /// Returns square AppKit checkbox (`NSButtonTypeSwitch`) tokens.
    pub fn checkbox(&self) -> &CheckboxMetrics { self.tokens.checkbox_metrics() }
    /// Returns non-editable AppKit label (`NSTextField.labelWithString:`) tokens.
    pub fn label_metrics(&self) -> &LabelMetrics { self.tokens.label_metrics() }
    /// Returns circular AppKit help button (`NSBezelStyleHelpButton`) tokens.
    pub fn help_button(&self) -> &HelpButtonMetrics { self.tokens.help_button_metrics() }
    /// Returns circular inline action badge button (`16x16pt`) tokens.
    pub fn inline_action_button(&self) -> &InlineActionButtonMetrics { self.tokens.inline_action_button_metrics() }
    /// Returns typography and SF Pro optical tracking tokens.
    pub fn typography(&self) -> &MacTypographyTokens { self.tokens.typography() }
    /// Returns CoreAnimation spring physics preset tokens.
    pub fn motion(&self) -> &MacMotionTokens { self.tokens.motion() }

    /// Evaluates [`MacBackgroundStyle`] for the given background surface color.
    pub fn background_style(&self, surface_bg: Color32) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(surface_bg)
    }
    /// Evaluates [`MacBackgroundStyle`] for [`Self::control_accent`] (or custom accent override).
    pub fn accent_background_style(&self) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(self.control_accent())
    }
    /// Evaluates [`MacBackgroundStyle`] for [`Self::selected_control`].
    pub fn selected_control_background_style(&self) -> MacBackgroundStyle {
        MacBackgroundStyle::from_color(self.selected_control())
    }
    /// Returns disabled control opacity factor (`0.45`).
    pub fn disabled_opacity(&self) -> f32 {
        self.tokens.control_metrics().disabled_opacity.points()
    }
    /// Scales `color` alpha by [`Self::disabled_opacity`] when `enabled` is `false`.
    pub fn apply_enabled(&self, color: Color32, enabled: bool) -> Color32 {
        if enabled { color } else { color.gamma_multiply(self.disabled_opacity()) }
    }

    /// Renders an AppKit outer keyboard focus ring (`NSFocusRingPlacementOutside`) around `rect`.
    pub fn paint_focus_ring(&self, painter: &egui::Painter, rect: egui::Rect, corner_radius: f32) {
        let ring = self.focus_ring();
        let focus_rect = rect.expand(ring.offset.points());
        let focus_radius = corner_radius + ring.corner_radius_delta.points();
        painter.rect_stroke(
            focus_rect,
            egui::CornerRadius::same(focus_radius as u8),
            egui::Stroke::new(ring.stroke_width.points(), self.keyboard_focus_indicator()),
            egui::StrokeKind::Outside,
        );
    }
}
