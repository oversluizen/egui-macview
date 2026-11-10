//! Authentic macOS AppKit Date Picker (`NSDatePicker` equivalent, REQ-012-02).
//!
//! Grounded in Apple AppKit `NSDatePicker` with integrated stepper, month calendar matrix,
//! and analog clock face across `Regular`, `Small`, and `Mini` control sizes.
//!
//! # Engineering Rationale & Apple AppKit Hardware Dimensions
//!
//! On macOS Sonoma 14.8.9 (Darwin 23.6.0), `NSDatePicker` presentation geometry adheres
//! to strict design invariants:
//!
//! - **Invariant 19.0 pt Stepper Width (`stepper_width`)**:
//!   In `NSDatePickerStyleTextFieldAndStepper`, the embedded right-anchored stepper is
//!   invariant at exactly $19.0\,\text{pt}$ across control sizes. This matches macOS
//!   Sonoma's standard hit-target budget for dual vertical increment/decrement arrows,
//!   bisected horizontally at the midpoint with a $9.0\,\text{pt}$ chevron pitch.
//!
//! - **139.0 x 148.0 pt Calendar Matrix Dimensions (`calendar_matrix_size`)**:
//!   In `NSDatePickerStyleClockAndCalendar`, the Month Calendar panel requires an exact
//!   $139.0 \times 148.0\,\text{pt}$ allocation derived from:
//!   - $24.0\,\text{pt}$ header bezel with month/year readout and month-flip buttons.
//!   - $16.0\,\text{pt}$ weekday initial banner ("S M T W T F S").
//!   - 6 rows $\times$ $18.0\,\text{pt}$ cell height ($108.0\,\text{pt}$) for calendar days.
//!   - Sum: $24.0 + 16.0 + 108.0 = 148.0\,\text{pt}$ total height.
//!   - 7 columns across $139.0\,\text{pt}$ ($\approx 19.85\,\text{pt}$ per cell column).

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
///
/// # Example
/// ```rust
/// use egui_macview::{
///     MacAppearance, MacDate, MacDatePicker, MacDatePickerElementFlags, MacDatePickerStyle,
///     MacTheme,
/// };
///
/// # let ctx = egui::Context::default();
/// # let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
/// let theme = MacTheme::new(MacAppearance::Aqua);
/// let mut date = MacDate::default();
///
/// ui.add(
///     MacDatePicker::new()
///         .date(&mut date)
///         .style(MacDatePickerStyle::TextFieldAndStepper)
///         .elements(MacDatePickerElementFlags::DateAndTime)
///         .theme(&theme),
/// );
/// # });
/// # out.textures_delta.clear();
/// ```
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
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.date_picker().stepper_width.points()
    }

    /// Vertical stepper chevron pitch in points ($9.0\,\text{pt}$).
    pub fn stepper_chevron_pitch(&self) -> f32 {
        let theme_cow = MacTheme::resolve_or_default(self.theme);
        theme_cow.date_picker().stepper_chevron_pitch.points()
    }

    /// Resolves exact dimensions for `NSDatePickerStyleTextFieldAndStepper` across flags and size.
    pub fn dimensions(flags: MacDatePickerElementFlags, size: MacControlSize) -> Vec2 {
        let theme = MacTheme::default();
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
        let theme = MacTheme::default();
        let m = theme.date_picker();
        Vec2::new(m.calendar_width.points(), m.calendar_height.points())
    }

    /// Resolves Analog Clock face bounding box dimensions ($122.0 \times 123.0\,\text{pt}$).
    pub fn clock_face_size() -> Vec2 {
        let theme = MacTheme::default();
        let m = theme.date_picker();
        Vec2::new(m.clock_face_width.points(), m.clock_face_height.points())
    }

    /// Resolves Analog Clock intrinsic circle dimensions ($119.0 \times 119.0\,\text{pt}$).
    pub fn clock_intrinsic_size() -> Vec2 {
        let theme = MacTheme::default();
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

        let (rect, mut response) = ui.allocate_exact_size(
            desired_size,
            Sense::click() | Sense::focusable_noninteractive(),
        );
        let rect = subpixel_snap_rect(rect, ui.ctx().pixels_per_point());

        let corner_r = match self.control_size {
            MacControlSize::Regular | MacControlSize::Large => metrics.corner_radius_regular.points(),
            MacControlSize::Small | MacControlSize::CompactInfobox => metrics.corner_radius_small.points(),
            MacControlSize::Mini => metrics.corner_radius_mini.points(),
        };

        let mut current_date = self.date.as_deref().copied().unwrap_or_default();
        let segment_key = response.id.with("active_segment");
        let mut active_segment = ui.data_mut(|d| d.get_temp::<usize>(segment_key).unwrap_or(0));

        let is_clicked = response.clicked()
            || (ui.input(|i| i.pointer.primary_clicked())
                && ui.input(|i| i.pointer.interact_pos()).is_some_and(|pos| rect.contains(pos)));
        if is_clicked {
            response.request_focus();
        }

        if !ui.is_rect_visible(rect) && !response.has_focus() {
            ui.data_mut(|d| d.insert_temp(segment_key, active_segment));
            if let Some(target_date) = self.date {
                *target_date = current_date;
            }
            return response;
        }

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

                // Stack-allocated segment tokens avoiding heap vectors and allocations
                #[derive(Clone, Copy)]
                struct SegToken {
                    idx: usize,
                    text: &'static str,
                    buf: [u8; 4],
                    len: u8,
                }
                impl SegToken {
                    const fn sep(s: &'static str) -> Self {
                        Self { idx: usize::MAX, text: s, buf: [0; 4], len: 0 }
                    }
                    fn num(idx: usize, val: u32, digits: usize) -> Self {
                        let mut buf = [0u8; 4];
                        let mut cur = val;
                        for i in (0..digits).rev() {
                            buf[i] = b'0' + (cur % 10) as u8;
                            cur /= 10;
                        }
                        Self { idx, text: "", buf, len: digits as u8 }
                    }
                    fn as_str(&self) -> &str {
                        if self.len > 0 {
                            std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
                        } else {
                            self.text
                        }
                    }
                }

                let mut seg_tokens = [SegToken::sep(""); 11];
                let count = match self.elements {
                    MacDatePickerElementFlags::DateAndTime => {
                        seg_tokens[0] = SegToken::num(0, current_date.year.max(0) as u32, 4);
                        seg_tokens[1] = SegToken::sep("-");
                        seg_tokens[2] = SegToken::num(1, current_date.month, 2);
                        seg_tokens[3] = SegToken::sep("-");
                        seg_tokens[4] = SegToken::num(2, current_date.day, 2);
                        seg_tokens[5] = SegToken::sep(" ");
                        seg_tokens[6] = SegToken::num(3, current_date.hour, 2);
                        seg_tokens[7] = SegToken::sep(":");
                        seg_tokens[8] = SegToken::num(4, current_date.minute, 2);
                        seg_tokens[9] = SegToken::sep(":");
                        seg_tokens[10] = SegToken::num(5, current_date.second, 2);
                        11
                    }
                    MacDatePickerElementFlags::DateOnly => {
                        seg_tokens[0] = SegToken::num(0, current_date.year.max(0) as u32, 4);
                        seg_tokens[1] = SegToken::sep("-");
                        seg_tokens[2] = SegToken::num(1, current_date.month, 2);
                        seg_tokens[3] = SegToken::sep("-");
                        seg_tokens[4] = SegToken::num(2, current_date.day, 2);
                        5
                    }
                    MacDatePickerElementFlags::TimeOnly => {
                        seg_tokens[0] = SegToken::num(0, current_date.hour, 2);
                        seg_tokens[1] = SegToken::sep(":");
                        seg_tokens[2] = SegToken::num(1, current_date.minute, 2);
                        seg_tokens[3] = SegToken::sep(":");
                        seg_tokens[4] = SegToken::num(2, current_date.second, 2);
                        5
                    }
                };

                let text_style = ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_default();
                let mut cur_x = rect.min.x + corner_r;
                let text_y = rect.center().y;

                for token in &seg_tokens[..count] {
                    let seg_idx = token.idx;
                    let is_active = seg_idx == active_segment;
                    let galley = painter.layout_no_wrap(
                        token.as_str().to_string(),
                        text_style.clone(),
                        if is_active { theme.on_accent_text() } else { theme.label() },
                    );
                    let seg_w = galley.size().x;
                    let seg_h = galley.size().y;
                    let seg_rect = Rect::from_min_size(
                        Pos2::new(cur_x, text_y - seg_h * 0.5),
                        Vec2::new(seg_w, seg_h),
                    );

                    let is_token_clicked = is_clicked
                        && ui.input(|i| i.pointer.interact_pos()).is_some_and(|pos| seg_rect.expand(1.0).contains(pos));
                    if is_token_clicked && seg_idx != usize::MAX {
                        active_segment = seg_idx;
                        response.mark_changed();
                        response.request_focus();
                    }

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

                if is_clicked {
                    response.request_focus();
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
