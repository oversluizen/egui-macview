//! Authentic macOS Calendar Matrix Grid and Analog Clock Face (SPEC-012, REQ-012-02).
//!
//! Grounded in Apple AppKit `NSDatePickerStyleClockAndCalendar` with Month Calendar
//! ($139.0 \times 148.0\,\text{pt}$) and Analog Clock Face ($122.0 \times 123.0\,\text{pt}$).
//!
//! # Separation of Concerns: Calendar Grid & Clock Face
//!
//! This module encapsulates all 2D calendar matrix math and clock hand rendering:
//! - **Zeller Calendar Grid**: Calculates weekday offsets via [`MacDate::first_weekday_of_month`],
//!   distributing 28..31 day cells across a strict 6-row $\times$ 7-column grid with
//!   $18.0\,\text{pt}$ cell pitch, header month stepping, and direct day selection.
//! - **Analog Clock Face**: Renders circular boundary ($119.0\,\text{pt}$ diameter), center pip,
//!   and trigonometry-driven hour and minute hands.

use crate::controls::date_picker::{MacDate, MacDatePicker, MacDatePickerElementFlags};
use crate::theme::MacTheme;
use egui::{CornerRadius, Pos2, Rect, Response, Stroke, Ui, Vec2};

/// Paints the Month Calendar Matrix grid and/or Analog Clock Face inside `rect`.
pub fn paint_clock_and_calendar(
    painter: &egui::Painter,
    rect: Rect,
    elements: MacDatePickerElementFlags,
    date: &mut MacDate,
    theme: &MacTheme,
    response: &mut Response,
    ui: &Ui,
) {
    let metrics = theme.date_picker();

    if elements == MacDatePickerElementFlags::DateOnly || elements == MacDatePickerElementFlags::DateAndTime {
        let cal_sz = MacDatePicker::calendar_matrix_size();
        let cal_rect = Rect::from_min_size(rect.min, cal_sz);
        let header_h = metrics.calendar_header_height.points();
        let weekday_h = metrics.calendar_weekday_height.points();
        let header_rect = Rect::from_min_size(cal_rect.min, Vec2::new(cal_rect.width(), header_h));

        // Month & year title
        let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let m_idx = (date.month.saturating_sub(1) as usize).min(11);
        let title = format!("{} {}", month_names[m_idx], date.year);
        let title_galley = painter.layout_no_wrap(
            title,
            ui.style().text_styles.get(&egui::TextStyle::Body).cloned().unwrap_or_default(),
            theme.label(),
        );
        painter.galley(
            Pos2::new(header_rect.center().x - title_galley.size().x * 0.5, header_rect.center().y - title_galley.size().y * 0.5),
            title_galley,
            theme.label(),
        );

        // Header navigation buttons
        let left_btn = Rect::from_min_size(header_rect.min, Vec2::new(header_h, header_h));
        let right_btn = Rect::from_min_size(Pos2::new(header_rect.max.x - header_h, header_rect.min.y), Vec2::new(header_h, header_h));
        painter.line_segment([left_btn.center(), Pos2::new(left_btn.center().x + 2.0, left_btn.center().y - 2.0)], Stroke::new(1.0, theme.label()));
        painter.line_segment([left_btn.center(), Pos2::new(left_btn.center().x + 2.0, left_btn.center().y + 2.0)], Stroke::new(1.0, theme.label()));
        painter.line_segment([right_btn.center(), Pos2::new(right_btn.center().x - 2.0, right_btn.center().y - 2.0)], Stroke::new(1.0, theme.label()));
        painter.line_segment([right_btn.center(), Pos2::new(right_btn.center().x - 2.0, right_btn.center().y + 2.0)], Stroke::new(1.0, theme.label()));

        if response.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if left_btn.contains(pos) {
                    if date.month <= 1 {
                        date.month = 12;
                        date.year = (date.year - 1).max(1);
                    } else {
                        date.month -= 1;
                    }
                    let max_d = date.days_in_month();
                    if date.day > max_d {
                        date.day = max_d;
                    }
                    response.mark_changed();
                } else if right_btn.contains(pos) {
                    if date.month >= 12 {
                        date.month = 1;
                        date.year += 1;
                    } else {
                        date.month += 1;
                    }
                    let max_d = date.days_in_month();
                    if date.day > max_d {
                        date.day = max_d;
                    }
                    response.mark_changed();
                }
            }
        }

        // Weekday header row (["S", "M", "T", "W", "T", "F", "S"])
        let cols_count = 7;
        let cell_w = cal_rect.width() / (cols_count as f32);
        let weekdays = ["S", "M", "T", "W", "T", "F", "S"];
        let small_style = ui.style().text_styles.get(&egui::TextStyle::Small).cloned().unwrap_or_default();
        for (c, &day_name) in weekdays.iter().enumerate() {
            let wk_rect = Rect::from_min_size(
                Pos2::new(cal_rect.min.x + (c as f32) * cell_w, cal_rect.min.y + header_h),
                Vec2::new(cell_w, weekday_h),
            );
            let g = painter.layout_no_wrap(day_name.to_string(), small_style.clone(), theme.secondary_label());
            painter.galley(
                Pos2::new(wk_rect.center().x - g.size().x * 0.5, wk_rect.center().y - g.size().y * 0.5),
                g,
                theme.secondary_label(),
            );
        }

        // Calendar days 6x7 matrix
        let grid_origin = Pos2::new(cal_rect.min.x, cal_rect.min.y + header_h + weekday_h);
        let cell_h = metrics.calendar_cell_height.points();
        let first_dow = date.first_weekday_of_month();
        let days_count = date.days_in_month();

        for d in 1..=days_count {
            let slot = first_dow + (d - 1) as usize;
            let col = slot % 7;
            let row = slot / 7;
            if row < 6 {
                let cell_rect = Rect::from_min_size(
                    Pos2::new(grid_origin.x + (col as f32) * cell_w, grid_origin.y + (row as f32) * cell_h),
                    Vec2::new(cell_w, cell_h),
                );
                let is_today = d == date.day;
                if is_today {
                    painter.rect_filled(
                        cell_rect.shrink(1.0),
                        CornerRadius::same(metrics.selection_corner_radius.points() as u8),
                        theme.control_accent(),
                    );
                }
                let text = format!("{d}");
                let g = painter.layout_no_wrap(
                    text,
                    ui.style().text_styles.get(&egui::TextStyle::Small).cloned().unwrap_or_default(),
                    if is_today { theme.on_accent_text() } else { theme.label() },
                );
                painter.galley(
                    Pos2::new(cell_rect.center().x - g.size().x * 0.5, cell_rect.center().y - g.size().y * 0.5),
                    g,
                    if is_today { theme.on_accent_text() } else { theme.label() },
                );

                if response.clicked() {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if cell_rect.contains(pos) {
                            date.day = d;
                            response.mark_changed();
                        }
                    }
                }
            }
        }
    }

    if elements == MacDatePickerElementFlags::TimeOnly || elements == MacDatePickerElementFlags::DateAndTime {
        let clk_origin = if elements == MacDatePickerElementFlags::DateAndTime {
            Pos2::new(rect.min.x + MacDatePicker::calendar_matrix_size().x + metrics.stepper_width.points(), rect.min.y)
        } else {
            rect.min
        };
        let clock_rect = Rect::from_min_size(clk_origin, MacDatePicker::clock_face_size());
        let clock_center = clock_rect.center();
        let radius = metrics.clock_size.points() * 0.5;

        painter.circle_stroke(clock_center, radius, Stroke::new(1.0, theme.separator()));

        // 12 subtle hour tick marks around perimeter (radius - 3.0 to radius)
        let clock_hours = 12;
        let clock_minutes = 60;
        let tick_len = metrics.corner_radius_mini.points();
        for h in 0..clock_hours {
            let angle = (h as f32) * (std::f32::consts::TAU / (clock_hours as f32)) - std::f32::consts::FRAC_PI_2;
            let (sin, cos) = angle.sin_cos();
            let p_outer = Pos2::new(clock_center.x + cos * radius, clock_center.y + sin * radius);
            let p_inner = Pos2::new(clock_center.x + cos * (radius - tick_len), clock_center.y + sin * (radius - tick_len));
            painter.line_segment([p_inner, p_outer], Stroke::new(1.0, theme.separator()));
        }

        painter.circle_filled(clock_center, 2.0, theme.control_accent());

        // Dynamic hour and minute hands
        let hr_val = (date.hour % clock_hours) as f32 + (date.minute as f32) / (clock_minutes as f32);
        let hr_angle = hr_val * (std::f32::consts::TAU / (clock_hours as f32)) - std::f32::consts::FRAC_PI_2;
        let min_angle = (date.minute as f32 / (clock_minutes as f32)) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;

        let hr_len = radius * 0.5;
        let min_len = radius * 0.5 + (radius * 0.5) * 0.5;

        let hr_tip = Pos2::new(clock_center.x + hr_angle.cos() * hr_len, clock_center.y + hr_angle.sin() * hr_len);
        painter.line_segment([clock_center, hr_tip], Stroke::new(2.0, theme.label()));

        let min_tip = Pos2::new(clock_center.x + min_angle.cos() * min_len, clock_center.y + min_angle.sin() * min_len);
        painter.line_segment([clock_center, min_tip], Stroke::new(1.0, theme.label()));

        // Interactive click-to-set-minute
        if response.clicked() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if clock_rect.contains(pos) {
                    let dx = pos.x - clock_center.x;
                    let dy = pos.y - clock_center.y;
                    let mut angle = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
                    if angle < 0.0 {
                        angle += std::f32::consts::TAU;
                    }
                    let minute_frac = angle / std::f32::consts::TAU;
                    let minute = ((minute_frac * (clock_minutes as f32)).round() as u32) % (clock_minutes as u32);
                    date.minute = minute;
                    response.mark_changed();
                }
            }
        }
    }
}
