//! Types and value representations for `MacDatePicker` (SPEC-012, REQ-012-02).

/// Presentation styles matching AppKit `NSDatePickerStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacDatePickerStyle {
    /// Text field with integrated right stepper (`NSDatePickerStyleTextFieldAndStepper`).
    #[default]
    TextFieldAndStepper,
    /// Visual clock and month calendar matrix grid (`NSDatePickerStyleClockAndCalendar`).
    ClockAndCalendar,
}

/// Element flags matching AppKit `NSDatePickerElementFlags`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MacDatePickerElementFlags {
    /// Date and time combined (`NSDatePickerElementFlagYearMonthDay | NSDatePickerElementFlagHourMinuteSecond`).
    #[default]
    DateAndTime,
    /// Date only (`NSDatePickerElementFlagYearMonthDay`).
    DateOnly,
    /// Time only (`NSDatePickerElementFlagHourMinuteSecond`).
    TimeOnly,
}

/// Date and time value representation for `MacDatePicker`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacDate {
    /// Calendar year (e.g. 2026).
    pub year: i32,
    /// Month index (1..=12).
    pub month: u32,
    /// Day of month (1..=31).
    pub day: u32,
    /// Hour (0..=23).
    pub hour: u32,
    /// Minute (0..=59).
    pub minute: u32,
    /// Second (0..=59).
    pub second: u32,
}

impl Default for MacDate {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 11,
            day: 10,
            hour: 10,
            minute: 14,
            second: 30,
        }
    }
}

impl MacDate {
    /// Returns number of days in the current month taking leap years into account.
    pub fn days_in_month(&self) -> u32 {
        match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                let y = self.year;
                if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Computes weekday index (0 = Sunday, 1 = Monday, ... 6 = Saturday) for the 1st of the month.
    pub fn first_weekday_of_month(&self) -> usize {
        let (y, m) = if self.month < 3 {
            (self.year - 1, self.month + 12)
        } else {
            (self.year, self.month)
        };
        let k = (y % 100) as usize;
        let j = (y / 100) as usize;
        let day = 1usize;
        let m = m as usize;
        let h = (day + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
        (h + 6) % 7
    }

    /// Increments or decrements a specific date/time segment with robust calendar invariants.
    pub fn step_segment(&mut self, segment: usize, flags: MacDatePickerElementFlags, delta: i32) {
        match (flags, segment) {
            (MacDatePickerElementFlags::DateAndTime, 0) | (MacDatePickerElementFlags::DateOnly, 0) => {
                self.year = (self.year + delta).clamp(1, 9999);
                let new_max = self.days_in_month();
                if self.day > new_max {
                    self.day = new_max;
                }
            }
            (MacDatePickerElementFlags::DateAndTime, 1) | (MacDatePickerElementFlags::DateOnly, 1) => {
                let m = ((self.month as i32 - 1 + delta).rem_euclid(12)) + 1;
                self.month = m as u32;
                let new_max = self.days_in_month();
                if self.day > new_max {
                    self.day = new_max;
                }
            }
            (MacDatePickerElementFlags::DateAndTime, 2) | (MacDatePickerElementFlags::DateOnly, 2) => {
                let max_days = self.days_in_month() as i32;
                let d = ((self.day as i32 - 1 + delta).rem_euclid(max_days)) + 1;
                self.day = d as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 3) | (MacDatePickerElementFlags::TimeOnly, 0) => {
                self.hour = ((self.hour as i32 + delta).rem_euclid(24)) as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 4) | (MacDatePickerElementFlags::TimeOnly, 1) => {
                self.minute = ((self.minute as i32 + delta).rem_euclid(60)) as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 5) | (MacDatePickerElementFlags::TimeOnly, 2) => {
                self.second = ((self.second as i32 + delta).rem_euclid(60)) as u32;
            }
            _ => {}
        }
    }
}
