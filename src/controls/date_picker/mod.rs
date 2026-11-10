//! Authentic macOS AppKit Date Picker (`NSDatePicker` equivalent, REQ-012-02).
//!
//! Grounded in Apple AppKit `NSDatePicker` with integrated stepper, month calendar matrix,
//! and analog clock face across `Regular`, `Small`, and `Mini` control sizes.

mod matrix;
mod types;

use crate::controls::MacControlSize;
use crate::layout::subpixel_snap_rect;
use crate::theme::MacTheme;
use egui::{
    CornerRadius, Key, Modifiers, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget,
};

pub use matrix::paint_clock_and_calendar;
pub use types::{MacDate, MacDatePickerElementFlags, MacDatePickerStyle};

/// Authentic macOS AppKit Date Picker (`NSDatePicker` equivalent).
pub struct MacDatePicker<'a> {
    date: Option<&'a mut MacDate>,
    style: MacDatePickerStyle,
    elements: MacDatePickerElementFlags,
    control_size: MacControlSize,
    theme: Option<&'a MacTheme>,
}

impl<'a> Default for MacDatePicker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MacDatePicker<'a> {
    /// Creates a new date picker widget.
    pub fn new() -> Self {
        Self {
            date: None,
            style: MacDatePickerStyle::TextFieldAndStepper,
            elements: MacDatePickerElementFlags::DateAndTime,
            control_size: MacControlSize::Regular,
            theme: None,
        }
    }

    /// Binds a mutable `MacDate` reference to the date picker.
    pub fn date(mut self, date: &'a mut MacDate) -> Self {
        self.date = Some(date);
        self
    }

    /// Sets the picker presentation style (`TextFieldAndStepper` or `ClockAndCalendar`).
    pub fn style(mut self, style: MacDatePickerStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets the date/time element flags (`DateAndTime`, `DateOnly`, or `TimeOnly`).
    pub fn elements(mut self, elements: MacDatePickerElementFlags) -> Self {
        self.elements = elements;
        self
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

    /// Stepper button width in points ($19.0\,\text{pt}$).
    pub fn stepper_width(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.date_picker().stepper_width.points()
    }

    /// Vertical stepper chevron pitch in points ($9.0\,\text{pt}$).
    pub fn stepper_chevron_pitch(&self) -> f32 {
        let theme_cow = MacTheme::resolve(self.theme, &egui::Context::default());
        theme_cow.date_picker().stepper_chevron_pitch.points()
    }

    /// Resolves exact dimensions for `NSDatePickerStyleTextFieldAndStepper` across flags and size.
    pub fn dimensions(flags: MacDatePickerElementFlags, size: MacControlSize) -> Vec2 {
        let theme = MacTheme::resolve(None, &egui::Context::default());
        let m = theme.date_picker();
        match (flags, size) {
            (MacDatePickerElementFlags::DateAndTime, MacControlSize::Regular | MacControlSize::Large) => {
                Vec2::new(m.datetime_regular_width.points(), m.datetime_regular_height.points())
            }
            (MacDatePickerElementFlags::DateAndTime, MacControlSize::Small | MacControlSize::CompactInfobox) => {
                Vec2::new(m.datetime_small_width.points(), m.datetime_small_height.points())
            }
            (MacDatePickerElementFlags::DateAndTime, MacControlSize::Mini) => {
                Vec2::new(m.datetime_mini_width.points(), m.datetime_mini_height.points())
            }
            (MacDatePickerElementFlags::DateOnly, MacControlSize::Regular | MacControlSize::Large) => {
                Vec2::new(m.date_only_regular_width.points(), m.date_only_regular_height.points())
            }
            (MacDatePickerElementFlags::DateOnly, MacControlSize::Small | MacControlSize::CompactInfobox) => {
                Vec2::new(m.date_only_small_width.points(), m.date_only_small_height.points())
            }
            (MacDatePickerElementFlags::DateOnly, MacControlSize::Mini) => {
                Vec2::new(m.date_only_mini_width.points(), m.date_only_mini_height.points())
            }
            (MacDatePickerElementFlags::TimeOnly, MacControlSize::Regular | MacControlSize::Large) => {
                Vec2::new(m.time_only_regular_width.points(), m.time_only_regular_height.points())
            }
            (MacDatePickerElementFlags::TimeOnly, MacControlSize::Small | MacControlSize::CompactInfobox) => {
                Vec2::new(m.time_only_small_width.points(), m.time_only_small_height.points())
            }
            (MacDatePickerElementFlags::TimeOnly, MacControlSize::Mini) => {
                Vec2::new(m.time_only_mini_width.points(), m.time_only_mini_height.points())
            }
        }
    }

    /// Resolves Month Calendar matrix grid dimensions ($139.0 \times 148.0\,\text{pt}$).
    pub fn calendar_matrix_size() -> Vec2 {
        let theme = MacTheme::resolve(None, &egui::Context::default());
        let m = theme.date_picker();
        Vec2::new(m.calendar_width.points(), m.calendar_height.points())
    }

    /// Resolves Analog Clock face bounding box dimensions ($122.0 \times 123.0\,\text{pt}$).
    pub fn clock_face_size() -> Vec2 {
        let theme = MacTheme::resolve(None, &egui::Context::default());
        let m = theme.date_picker();
        Vec2::new(m.clock_face_width.points(), m.clock_face_height.points())
    }

    /// Resolves Analog Clock intrinsic circle dimensions ($119.0 \times 119.0\,\text{pt}$).
    pub fn clock_intrinsic_size() -> Vec2 {
        let theme = MacTheme::resolve(None, &egui::Context::default());
        let sz = theme.date_picker().clock_size.points();
        Vec2::new(sz, sz)
    }
}

impl<'a> Widget for MacDatePicker<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme_cow = MacTheme::resolve(self.theme, ui.ctx());
        let theme: &MacTheme = &theme_cow;
        let metrics = theme.date_picker();

        let desired_size = match self.style {
            MacDatePickerStyle::TextFieldAndStepper => Self::dimensions(self.elements, self.control_size),
            MacDatePickerStyle::ClockAndCalendar => match self.elements {
                MacDatePickerElementFlags::DateOnly => Self::calendar_matrix_size(),
                MacDatePickerElementFlags::TimeOnly => Self::clock_face_size(),
                MacDatePickerElementFlags::DateAndTime => {
                    let cal = Self::calendar_matrix_size();
                    let clk = Self::clock_face_size();
                    Vec2::new(cal.x + metrics.stepper_width.points() + clk.x, cal.y.max(clk.y))
                }
            },
        };

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let corner_r = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => metrics.corner_radius_regular.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => metrics.corner_radius_small.points(),
            MacControlSize::Mini => metrics.corner_radius_mini.points(),
        };

        let mut current_date = self.date.as_deref().copied().unwrap_or_default();
        let segment_key = response.id.with("active_segment");
        let mut active_segment = ui.data_mut(|d| d.get_temp::<usize>(segment_key).unwrap_or(0));

        let painter = ui.painter_at(rect);

        match self.style {
            MacDatePickerStyle::TextFieldAndStepper => {
                painter.rect(
                    rect,
                    CornerRadius::same(corner_r as u8),
                    theme.control_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                let st_w = metrics.stepper_width.points();
                let stepper_rect = Rect::from_min_max(
                    Pos2::new(rect.max.x - st_w, rect.min.y),
                    rect.max,
                );

                painter.line_segment(
                    [
                        Pos2::new(stepper_rect.min.x, rect.min.y),
                        Pos2::new(stepper_rect.min.x, rect.max.y),
                    ],
                    Stroke::new(1.0, theme.separator()),
                );

                let cx = stepper_rect.center().x;
                let cy = stepper_rect.center().y;
                let pitch = metrics.stepper_chevron_pitch.points() * 0.5;
                let ch_w = (st_w * 0.5) * 0.5;

                // Stepper chevrons
                painter.line_segment(
                    [Pos2::new(cx - ch_w, cy - pitch + 1.0), Pos2::new(cx, cy - pitch - 1.0)],
                    Stroke::new(1.0, theme.secondary_label()),
                );
                painter.line_segment(
                    [Pos2::new(cx, cy - pitch - 1.0), Pos2::new(cx + ch_w, cy - pitch + 1.0)],
                    Stroke::new(1.0, theme.secondary_label()),
                );
                painter.line_segment(
                    [Pos2::new(cx - ch_w, cy + pitch - 1.0), Pos2::new(cx, cy + pitch + 1.0)],
                    Stroke::new(1.0, theme.secondary_label()),
                );
                painter.line_segment(
                    [Pos2::new(cx, cy + pitch + 1.0), Pos2::new(cx + ch_w, cy + pitch - 1.0)],
                    Stroke::new(1.0, theme.secondary_label()),
                );

                // Build segment tokens
                let segments: Vec<(usize, String)> = match self.elements {
                    MacDatePickerElementFlags::DateAndTime => vec![
                        (0, format!("{:04}", current_date.year)),
                        (usize::MAX, "-".to_string()),
                        (1, format!("{:02}", current_date.month)),
                        (usize::MAX, "-".to_string()),
                        (2, format!("{:02}", current_date.day)),
                        (usize::MAX, " ".to_string()),
                        (3, format!("{:02}", current_date.hour)),
                        (usize::MAX, ":".to_string()),
                        (4, format!("{:02}", current_date.minute)),
                        (usize::MAX, ":".to_string()),
                        (5, format!("{:02}", current_date.second)),
                    ],
                    MacDatePickerElementFlags::DateOnly => vec![
                        (0, format!("{:04}", current_date.year)),
                        (usize::MAX, "-".to_string()),
                        (1, format!("{:02}", current_date.month)),
                        (usize::MAX, "-".to_string()),
                        (2, format!("{:02}", current_date.day)),
                    ],
                    MacDatePickerElementFlags::TimeOnly => vec![
                        (0, format!("{:02}", current_date.hour)),
                        (usize::MAX, ":".to_string()),
                        (1, format!("{:02}", current_date.minute)),
                        (usize::MAX, ":".to_string()),
                        (2, format!("{:02}", current_date.second)),
                    ],
                };

                let text_style = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_default();
                let mut cur_x = rect.min.x + corner_r;
                let text_y = rect.center().y;

                for (seg_idx, text) in &segments {
                    let is_active = *seg_idx == active_segment;
                    let galley = painter.layout_no_wrap(
                        text.clone(),
                        text_style.clone(),
                        if is_active { theme.on_accent_text() } else { theme.label() },
                    );
                    let seg_w = galley.size().x;
                    let seg_h = galley.size().y;
                    let seg_rect = Rect::from_min_size(
                        Pos2::new(cur_x, text_y - seg_h * 0.5),
                        Vec2::new(seg_w, seg_h),
                    );

                    if is_active {
                        painter.rect_filled(
                            seg_rect.expand(1.0),
                            CornerRadius::same(metrics.selection_corner_radius.points() as u8),
                            theme.selected_control(),
                        );
                    }

                    painter.galley(
                        Pos2::new(cur_x, text_y - seg_h * 0.5),
                        galley,
                        if is_active { theme.on_accent_text() } else { theme.label() },
                    );
                    cur_x += seg_w;
                }

                // Handle stepper click & arrow keys
                let top_half = Rect::from_min_max(stepper_rect.min, Pos2::new(stepper_rect.max.x, cy));
                let bottom_half = Rect::from_min_max(Pos2::new(stepper_rect.min.x, cy), stepper_rect.max);

                if response.clicked() {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if top_half.contains(pos) {
                            current_date.step_segment(active_segment, self.elements, 1);
                            response.mark_changed();
                        } else if bottom_half.contains(pos) {
                            current_date.step_segment(active_segment, self.elements, -1);
                            response.mark_changed();
                        }
                    }
                }

                if response.has_focus() {
                    let max_seg = match self.elements {
                        MacDatePickerElementFlags::DateAndTime => 5,
                        _ => 2,
                    };
                    if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                        current_date.step_segment(active_segment, self.elements, 1);
                        response.mark_changed();
                    } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                        current_date.step_segment(active_segment, self.elements, -1);
                        response.mark_changed();
                    } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
                        active_segment = active_segment.saturating_sub(1);
                    } else if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::ArrowRight)) {
                        active_segment = (active_segment + 1).min(max_seg);
                    }
                }
            }
            MacDatePickerStyle::ClockAndCalendar => {
                painter.rect(
                    rect,
                    CornerRadius::same(corner_r as u8),
                    theme.control_background(),
                    Stroke::new(1.0, theme.separator()),
                    StrokeKind::Inside,
                );

                matrix::paint_clock_and_calendar(
                    &painter,
                    rect,
                    self.elements,
                    &mut current_date,
                    theme,
                    &mut response,
                    ui,
                );
            }
        }

        ui.data_mut(|d| d.insert_temp(segment_key, active_segment));

        if let Some(target_date) = self.date {
            *target_date = current_date;
        }

        response
    }
}
