//! macOS AppKit Authentic Controls for egui.

use egui::{Color32, CornerRadius, Pos2, Rect, Response, Stroke, StrokeKind, Ui, Vec2, Widget};

/// Authentic macOS compact text entry cell (default 18pt height, AppKit NSTextFieldCell style).
pub struct MacInputCell<'a> {
    id: egui::Id,
    value: &'a mut String,
    width: f32,
    height: f32,
    placeholder: Option<&'a str>,
}

impl<'a> MacInputCell<'a> {
    pub fn new(id_source: impl std::hash::Hash + std::fmt::Debug, value: &'a mut String) -> Self {
        #[allow(deprecated)]
        let id = egui::Id::new(id_source);
        Self {
            id,
            value,
            width: 44.0,
            height: 18.0,
            placeholder: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = Some(text);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(self.width, self.height);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        let is_focused = ui.memory(|m| m.has_focus(self.id));
        let painter = ui.painter_at(rect);

        // macOS AppKit NSTextField border & background
        let bg_color = if ui.visuals().dark_mode {
            Color32::from_rgb(30, 30, 30)
        } else {
            Color32::WHITE
        };
        let border_stroke = if is_focused {
            Stroke::new(1.5, Color32::from_rgb(0, 122, 255))
        } else if response.hovered() {
            Stroke::new(1.0, Color32::from_rgb(180, 180, 180))
        } else {
            Stroke::new(1.0, Color32::from_rgb(210, 210, 210))
        };

        painter.rect(rect, CornerRadius::same(3), bg_color, border_stroke, StrokeKind::Inside);

        // Inner text edit
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink(2.0))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let text_res = child_ui.add(
            egui::TextEdit::singleline(self.value)
                .id(self.id)
                .font(egui::FontId::proportional(11.0))
                .desired_width(self.width - 4.0)
                .frame(egui::Frame::NONE),
        );

        if text_res.changed() {
            response.mark_changed();
        }

        response
    }
}

/// Authentic macOS AppKit Push Button / Tool Button.
pub struct MacButtonCell<'a> {
    text: &'a str,
    selected: bool,
    width: Option<f32>,
    height: f32,
}

impl<'a> MacButtonCell<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            selected: false,
            width: None,
            height: 20.0,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let padding = Vec2::new(8.0, 0.0);
        let font_id = egui::FontId::proportional(12.0);
        let text_size = ui.painter().layout_no_wrap(
            self.text.to_string(),
            font_id.clone(),
            Color32::BLACK,
        ).size();

        let w = self.width.unwrap_or(text_size.x + padding.x * 2.0);
        let desired_size = Vec2::new(w, self.height);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        let painter = ui.painter_at(rect);
        let is_hovered = response.hovered();
        let is_active = response.is_pointer_button_down_on();

        let bg = if self.selected {
            Color32::from_rgb(0, 122, 255)
        } else if is_active {
            Color32::from_rgb(200, 200, 200)
        } else if is_hovered {
            Color32::from_rgb(230, 230, 230)
        } else {
            Color32::from_rgb(245, 245, 245)
        };

        let stroke = if self.selected {
            Stroke::NONE
        } else {
            Stroke::new(1.0, Color32::from_rgb(215, 215, 215))
        };

        painter.rect(rect, CornerRadius::same(4), bg, stroke, StrokeKind::Inside);

        let text_color = if self.selected {
            Color32::WHITE
        } else {
            Color32::from_rgb(30, 30, 30)
        };

        let text_pos = Pos2::new(
            rect.center().x - text_size.x * 0.5,
            rect.center().y - text_size.y * 0.5,
        );
        painter.text(text_pos, egui::Align2::LEFT_TOP, self.text, font_id, text_color);

        response
    }
}

/// Authentic macOS NSSegmentedControl.
pub struct MacSegmentedControl<'a> {
    options: &'a [&'a str],
    selected_idx: usize,
    height: f32,
}

impl<'a> MacSegmentedControl<'a> {
    pub fn new(options: &'a [&'a str], selected_idx: usize) -> Self {
        Self {
            options,
            selected_idx,
            height: 20.0,
        }
    }

    pub fn show(self, ui: &mut Ui, on_select: &mut impl FnMut(usize)) -> Response {
        let item_width = 32.0;
        let total_w = item_width * self.options.len() as f32;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(total_w, self.height), egui::Sense::click());

        let painter = ui.painter_at(rect);
        // Outer capsule background
        painter.rect(
            rect,
            CornerRadius::same(5),
            Color32::from_rgb(235, 235, 235),
            Stroke::new(1.0, Color32::from_rgb(210, 210, 210)),
            StrokeKind::Inside,
        );

        // Active segment capsule
        let active_rect = Rect::from_min_size(
            Pos2::new(rect.min.x + self.selected_idx as f32 * item_width + 1.0, rect.min.y + 1.0),
            Vec2::new(item_width - 2.0, self.height - 2.0),
        );
        painter.rect(
            active_rect,
            CornerRadius::same(4),
            Color32::WHITE,
            Stroke::new(0.5, Color32::from_rgb(190, 190, 190)),
            StrokeKind::Inside,
        );

        let font_id = egui::FontId::proportional(11.0);
        for (i, opt) in self.options.iter().enumerate() {
            let seg_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * item_width, rect.min.y),
                Vec2::new(item_width, self.height),
            );
            if response.clicked() {
                if let Some(pos) = ui.input(|inp| inp.pointer.interact_pos()) {
                    if seg_rect.contains(pos) {
                        on_select(i);
                    }
                }
            }
            let color = if i == self.selected_idx {
                Color32::from_rgb(20, 20, 20)
            } else {
                Color32::from_rgb(110, 110, 110)
            };
            painter.text(seg_rect.center(), egui::Align2::CENTER_CENTER, *opt, font_id.clone(), color);
        }

        response
    }
}

/// Authentic macOS Toggle / Switch.
pub struct MacToggle<'a> {
    value: &'a mut bool,
}

impl<'a> MacToggle<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self { value }
    }
}

impl<'a> Widget for MacToggle<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(32.0, 18.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        if response.clicked() {
            *self.value = !*self.value;
            response.mark_changed();
        }

        let painter = ui.painter_at(rect);
        let bg_color = if *self.value {
            Color32::from_rgb(52, 199, 89) // macOS Green
        } else {
            Color32::from_rgb(220, 220, 220)
        };

        painter.rect(rect, CornerRadius::same(9), bg_color, Stroke::NONE, StrokeKind::Inside);

        let knob_x = if *self.value {
            rect.max.x - 9.0
        } else {
            rect.min.x + 9.0
        };
        let knob_center = Pos2::new(knob_x, rect.center().y);
        painter.circle_filled(knob_center, 7.5, Color32::WHITE);

        response
    }
}
