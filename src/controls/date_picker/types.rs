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

    /// Increments or decrements a specific date/time segment.
    pub fn step_segment(&mut self, segment: usize, flags: MacDatePickerElementFlags, delta: i32) {
        let max_days = self.days_in_month() as i32;
        match (flags, segment) {
            (MacDatePickerElementFlags::DateAndTime, 0) | (MacDatePickerElementFlags::DateOnly, 0) => {
                self.year += delta;
            }
            (MacDatePickerElementFlags::DateAndTime, 1) | (MacDatePickerElementFlags::DateOnly, 1) => {
                let mut m = self.month as i32 + delta;
                if m < 1 {
                    m = 12;
                } else if m > 12 {
                    m = 1;
                }
                self.month = m as u32;
                let new_max = self.days_in_month();
                if self.day > new_max {
                    self.day = new_max;
                }
            }
            (MacDatePickerElementFlags::DateAndTime, 2) | (MacDatePickerElementFlags::DateOnly, 2) => {
                let mut d = self.day as i32 + delta;
                if d < 1 {
                    d = max_days;
                } else if d > max_days {
                    d = 1;
                }
                self.day = d as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 3) | (MacDatePickerElementFlags::TimeOnly, 0) => {
                let mut h = self.hour as i32 + delta;
                if h < 0 {
                    h = 23;
                } else if h > 23 {
                    h = 0;
                }
                self.hour = h as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 4) | (MacDatePickerElementFlags::TimeOnly, 1) => {
                let mut min = self.minute as i32 + delta;
                if min < 0 {
                    min = 59;
                } else if min > 59 {
                    min = 0;
                }
                self.minute = min as u32;
            }
            (MacDatePickerElementFlags::DateAndTime, 5) | (MacDatePickerElementFlags::TimeOnly, 2) => {
                let mut s = self.second as i32 + delta;
                if s < 0 {
                    s = 59;
                } else if s > 59 {
                    s = 0;
                }
                self.second = s as u32;
            }
            _ => {}
        }
    }
}
