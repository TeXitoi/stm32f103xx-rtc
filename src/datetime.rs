#[derive(Debug)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl DayOfWeek {
    pub fn from_days_since_epoch(days: u32) -> DayOfWeek {
        use self::DayOfWeek::*;
        match days % 7 {
            4 => Monday,
            5 => Tuesday,
            6 => Wednesday,
            0 => Thursday,
            1 => Friday,
            2 => Saturday,
            3 => Sunday,
            _ => unreachable!(),
        }
    }
}
impl ::core::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub day_of_week: DayOfWeek,
}
impl DateTime {
    fn is_leap(year: u16) -> bool {
        if year % 4 != 0 {
            false
        } else if year % 100 != 0 {
            true
        } else {
            year % 400 == 0
        }
    }
    pub fn new(epoch: u32) -> DateTime {
        let mut days = epoch / 86400;
        let time = epoch % 86400;
        let day_of_week = DayOfWeek::from_days_since_epoch(days);
        let mut year = 1970;
        let mut is_leap;

        loop {
            is_leap = Self::is_leap(year);
            if is_leap && days > 366 {
                year += 1;
                days -= 366
            } else if !is_leap && days > 365 {
                year += 1;
                days -= 365;
            } else {
                break;
            }
        }
        let mut days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if is_leap { days_in_month[2] = 29; }
        let mut month = 1;
        for &nb in days_in_month.iter() {
            if days < nb { break; }
            days -= nb;
            month += 1;
        }
        DateTime {
            year: year,
            month: month,
            day: (days + 1) as u8,
            hour: (time / 60 / 60) as u8,
            min: (time / 60 % 60) as u8,
            sec: (time % 60) as u8,
            day_of_week,
        }
    }
}
impl ::core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} ({})",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.min,
            self.sec,
            self.day_of_week,
        )
    }
}
