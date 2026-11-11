//! Authentic macOS AppKit Combo Box (`NSComboBox` equivalent, REQ-012-03 & v0.6.5).

mod popover;

pub use popover::ComboBoxPopoverGeometry;

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, FontId, Id, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    TextEdit, Ui, UiBuilder, Vec2,
};
use std::hash::Hash;

/// Computed layout geometry for the combo box text field and trigger button header.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComboBoxHeaderLayout {
    /// Outer bounding rectangle.
    pub total_rect: Rect,
    /// Bounding rectangle for the right-hand dropdown trigger button.
    pub button_rect: Rect,
    /// Bounding rectangle for the editable text input field.
    pub text_rect: Rect,
    /// Resolved corner radius.
    pub corner_radius: f32,
}

/// Authentic macOS AppKit Combo Box (`NSComboBox` equivalent).
///
/// # Example
/// ```rust
/// use egui_macview::{MacAppearance, MacComboBox, MacTheme};
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut font_name = String::from("SF Pro Text");
/// let fonts = ["Helvetica Neue", "Menlo", "SF Mono", "SF Pro Display", "SF Pro Text"];
/// MacComboBox::new("font_combo", &mut font_name, &fonts).width(180.0).theme(&theme).show(ui);
/// # });
/// # out.textures_delta.clear();
/// ```
pub struct MacComboBox<'a, T: AsRef<str>> {
    pub(super) id_salt: Id,
    pub(super) text: &'a mut String,
    pub(super) items: &'a [T],
    pub(super) control_size: MacControlSize,
    pub(super) theme: Option<&'a MacTheme>,
    pub(super) width: Option<f32>,
    pub(super) fixed_rect: Option<Rect>,
}

impl<'a, T: AsRef<str>> MacComboBox<'a, T> {
    /// Creates a new combo box for an item slice and a mutable text buffer.
    pub fn new(id_salt: impl Hash + std::fmt::Debug, text: &'a mut String, items: &'a [T]) -> Self {
        Self {
            id_salt: Id::NULL.with(id_salt),
            text,
            items,
            control_size: MacControlSize::Regular,
            theme: None,
            width: None,
            fixed_rect: None,
        }
    }

    /// Sets the control size (`Regular`, `Small`, or `Mini`).
    pub fn control_size(mut self, size: MacControlSize) -> Self {
        self.control_size = size;
        self
    }

    /// Injects dynamic theme.
    pub fn theme(mut self, theme: &'a MacTheme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Overrides default width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Pins the combo box to an exact NIB rectangle.
    pub fn fixed_rect(mut self, rect: Rect) -> Self {
        self.fixed_rect = Some(rect);
        self
    }

    /// Resolves outer frame height in points.
    pub fn frame_height(&self) -> f32 {
        let t = MacTheme::resolve_or_default(self.theme);
        let m = t.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_height.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_height.points(),
            MacControlSize::Mini => m.mini_height.points(),
        }
    }

    /// Resolves visual bezel height in points (`20.0pt` Regular, `16.0pt` Small, `13.0pt` Mini).
    pub fn visual_height(&self) -> f32 {
        let t = MacTheme::resolve_or_default(self.theme);
        let m = t.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_visual_height.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_visual_height.points(),
            MacControlSize::Mini => m.mini_visual_height.points(),
        }
    }

    /// Resolves right dropdown trigger button width in points.
    pub fn button_width(&self) -> f32 {
        let t = MacTheme::resolve_or_default(self.theme);
        let m = t.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_button_width.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_button_width.points(),
            MacControlSize::Mini => m.mini_button_width.points(),
        }
    }

    /// Resolves left text padding inset in points.
    pub fn text_inset(&self) -> f32 {
        let t = MacTheme::resolve_or_default(self.theme);
        let m = t.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_inset.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_inset.points(),
            MacControlSize::Mini => m.mini_inset.points(),
        }
    }

    /// Resolves corner radius in points.
    pub fn corner_radius(&self) -> f32 {
        let t = MacTheme::resolve_or_default(self.theme);
        let m = t.combo_box();
        match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => m.regular_corner_radius.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => m.small_corner_radius.points(),
            MacControlSize::Mini => m.mini_corner_radius.points(),
        }
    }

    /// Resolves popover single item row height in points ($18.0\,\text{pt}$).
    pub fn item_height(&self) -> f32 {
        MacTheme::resolve_or_default(self.theme).combo_box().item_height.points()
    }

    /// Resolves visible items count before scrolling ($5$).
    pub fn visible_items(&self) -> usize {
        MacTheme::resolve_or_default(self.theme).combo_box().visible_items.points() as usize
    }

    /// Resolves popover viewport height before scrolling ($90.0\,\text{pt}$).
    pub fn popover_viewport_height(&self) -> f32 {
        self.item_height() * (self.visible_items() as f32)
    }

    /// Phase 1: Computes the layout rectangles of the outer frame, text area, and trigger button.
    pub fn compute_header_layout(&self, ui: &Ui) -> ComboBoxHeaderLayout {
        let (height, btn_w, corner_r) = (self.frame_height(), self.button_width(), self.corner_radius());
        let desired_w = self.fixed_rect.map(|r| r.width()).or(self.width)
            .unwrap_or_else(|| ui.available_width().max(btn_w * 2.0));
        let raw = self.fixed_rect.unwrap_or_else(|| Rect::from_min_size(ui.cursor().min, Vec2::new(desired_w, height)));
        let rect = subpixel_snap_rect(raw, ui.ctx().pixels_per_point());
        let text_inset = self.text_inset();
        ComboBoxHeaderLayout {
            total_rect: rect,
            button_rect: Rect::from_min_max(Pos2::new(rect.max.x - btn_w, rect.min.y), rect.max),
            text_rect: Rect::from_min_max(Pos2::new(rect.min.x + text_inset, rect.min.y), Pos2::new(rect.max.x - btn_w, rect.max.y)),
            corner_radius: corner_r,
        }
    }

    /// Phase 2: Computes popover dimensions and screen-edge boundary clamping.
    pub fn compute_popover_geometry(&self, ui: &Ui, header_rect: Rect, item_count: usize) -> ComboBoxPopoverGeometry {
        popover::compute_popover_geometry(self, ui, header_rect, item_count)
    }

    /// Shows the combo box widget.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.combo_box();
        let layout = self.compute_header_layout(ui);
        let enabled = ui.is_enabled();
        let sense = if enabled { Sense::click() } else { Sense::hover() };
        let ppp = ui.ctx().pixels_per_point();
        let (frame_rect, visual_rect, mut response) = if let Some(fixed) = self.fixed_rect {
            let sf = subpixel_snap_rect(fixed, ppp);
            let resp = ui.interact(sf, self.id_salt.with("fixed_combo"), sense);
            (sf, theme.clamp_nib_visual_rect(sf, self.visual_height(), ppp), resp)
        } else {
            let (r, resp) = ui.allocate_exact_size(layout.total_rect.size(), sense);
            let snapped = subpixel_snap_rect(r, ppp);
            (snapped, snapped, resp)
        };

        let open_id = self.id_salt.with("popover_open");
        let mut is_open = enabled && ui.data_mut(|d| d.get_temp::<bool>(open_id).unwrap_or(false));
        let selected_item_id = self.id_salt.with("popover_sel_idx");
        let stored_sel_idx = ui.data_mut(|d| d.get_temp::<usize>(selected_item_id));

        let (btn_w, text_inset) = (self.button_width(), self.text_inset());
        let btn_rect = Rect::from_min_max(Pos2::new(visual_rect.max.x - btn_w, visual_rect.min.y), visual_rect.max);
        let text_rect = Rect::from_min_max(
            Pos2::new(visual_rect.min.x + text_inset, visual_rect.min.y),
            Pos2::new(visual_rect.max.x - btn_w, visual_rect.max.y),
        );

        if enabled && (ui.input_mut(|i| i.consume_key(Modifiers::ALT, Key::ArrowDown))
            || ui.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowDown)))
        {
            is_open = true;
        }

        let painter = ui.painter_at(frame_rect);
        let border_col = if enabled && response.has_focus() {
            theme.control_accent()
        } else {
            theme.apply_enabled(theme.separator(), enabled)
        };
        let bg_fill = theme.apply_enabled(theme.control_background(), enabled);
        painter.rect(visual_rect, CornerRadius::same(layout.corner_radius as u8), bg_fill, Stroke::new(1.0, border_col), StrokeKind::Inside);
        painter.line_segment([Pos2::new(btn_rect.min.x, visual_rect.min.y), Pos2::new(btn_rect.min.x, visual_rect.max.y)], Stroke::new(1.0, theme.apply_enabled(theme.separator(), enabled)));

        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let btn_hovered = enabled && pointer_pos.is_some_and(|p| btn_rect.contains(p));
        let btn_down = btn_hovered && ui.input(|i| i.pointer.primary_down());
        let btn_radius = CornerRadius { nw: 0, sw: 0, ne: layout.corner_radius as u8, se: layout.corner_radius as u8 };
        if btn_down {
            painter.rect_filled(btn_rect, btn_radius, theme.selected_control());
        } else if btn_hovered {
            painter.rect_filled(btn_rect, btn_radius, theme.control());
        }
        if btn_hovered && ui.input(|i| i.pointer.primary_clicked()) {
            is_open = !is_open;
        }

        // Right accent button badge (`NSComboBoxCell` 17pt badge, 2pt inset, 4pt radius)
        let (badge_w, badge_inset, badge_r) = (metrics.badge_button_width.points(), metrics.button_inset.points(), metrics.button_corner_radius.points());
        let badge_rect = Rect::from_min_max(
            Pos2::new((visual_rect.max.x - badge_inset - badge_w).max(btn_rect.min.x), visual_rect.min.y + badge_inset),
            Pos2::new(visual_rect.max.x - badge_inset, visual_rect.max.y - badge_inset),
        );
        let badge_fill = if !enabled { theme.control().gamma_multiply(theme.disabled_opacity()) } else if btn_down { theme.selected_control() } else { theme.control_accent() };
        painter.rect_filled(badge_rect, CornerRadius::same(badge_r as u8), badge_fill);

        let (cx, cy) = (badge_rect.center().x, badge_rect.center().y);
        let (ch_hw, ch_hh) = (metrics.chevron_half_width.points(), metrics.chevron_half_height.points());
        let ch_stroke = Stroke::new(metrics.chevron_stroke_width.points(), theme.apply_enabled(theme.on_accent_text(), enabled));
        painter.line_segment([Pos2::new(cx - ch_hw, cy - ch_hh * 0.5), Pos2::new(cx, cy + ch_hh)], ch_stroke);
        painter.line_segment([Pos2::new(cx, cy + ch_hh), Pos2::new(cx + ch_hw, cy - ch_hh * 0.5)], ch_stroke);

        let mut child_ui = ui.new_child(UiBuilder::new().id_salt(self.id_salt.with("text_edit")).max_rect(text_rect));
        let font_sz = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => theme.text_field().regular.font_size.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => theme.text_field().small.font_size.points(),
            MacControlSize::Mini => theme.text_field().mini.font_size.points(),
        };
        let edit_id = self.id_salt.with("combo_input_id");
        let is_edit_focused = enabled && ui.memory(|m| m.has_focus(edit_id));
        if is_edit_focused {
            let edit_res = child_ui.add_sized(
                text_rect.size(),
                TextEdit::singleline(self.text).id(edit_id).font(FontId::proportional(font_sz)).text_color(theme.label()).frame(egui::Frame::NONE),
            );
            if edit_res.changed() {
                response.mark_changed();
                is_open = true;
            }
        } else {
            let click_res = child_ui.interact(text_rect, edit_id, sense);
            if enabled && click_res.clicked() {
                ui.memory_mut(|m| m.request_focus(edit_id));
            }
            if !self.text.is_empty() {
                let lm = theme.label_metrics();
                let (baseline_from_top, ascender) = match self.control_size {
                    MacControlSize::Regular | MacControlSize::Large => (metrics.regular_first_baseline_offset.points(), lm.regular_ascender.points()),
                    MacControlSize::Small | MacControlSize::CompactInfobox => (metrics.small_first_baseline_offset.points(), lm.small_ascender.points()),
                    MacControlSize::Mini => (metrics.mini_first_baseline_offset.points(), lm.mini_ascender.points()),
                };
                let text_col = theme.apply_enabled(theme.label(), enabled);
                let galley = theme.layout_tracked_galley(ui, self.text.as_str(), FontId::proportional(font_sz), text_col, text_rect.width().max(0.0));
                let text_y = theme.baseline_text_y(visual_rect, baseline_from_top, ascender, ppp);
                painter.with_clip_rect(text_rect).galley(Pos2::new(text_rect.left(), text_y), galley, text_col);
            }
        }

        popover::handle_combo_popover(
            &mut self, ui, theme, visual_rect, layout.corner_radius, &mut is_open,
            stored_sel_idx, open_id, selected_item_id, pointer_pos, &mut response,
        );
        response
    }
}
