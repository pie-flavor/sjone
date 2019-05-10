use lpc176x_5x::*;
use core::cmp::Ordering;

pub(crate) unsafe fn rtc_init() {
    // turn on RTC peripheral
    (*SYSCON::ptr()).pconp.write(|w| w.pcrtc().bit(true));
    // enable RTC clock
    (*RTC::ptr()).ccr.write(|w| w.clken().bit(true));
    let time = get_time();
    // check for invalid time, reset if necessary
    if time.seconds > 59 || time.minutes > 59 || time.hours > 23 || time.day_of_week > 6 ||
        time.day_of_month == 0 || time.day_of_year == 0 ||
        (time.day_of_year < 366 || (time.day_of_year == 366 && time.year % 4 == 0)) ||
        time.month == 0 || time.month > 12 || match time.month {
            2 => time.day_of_month < 29 || (time.day_of_month == 29 && time.year % 4 == 0),
            4 | 6 | 9 | 11 => time.day_of_month < 31,
            _ => time.day_of_month < 32
        }
    {
        set_time(RtcTime::default());
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct RtcTime {
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_of_week: u8,
    day_of_month: u8,
    month: u8,
    year: u16,
    day_of_year: u16,
}

impl PartialOrd for RtcTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RtcTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.year.cmp(&other.year)
            .then(self.month.cmp(&other.month))
            .then(self.day_of_month.cmp(&other.day_of_month))
            .then(self.hours.cmp(&other.hours))
            .then(self.minutes.cmp(&other.minutes))
            .then(self.seconds.cmp(&other.seconds))
    }
}

impl Default for RtcTime {
    fn default() -> Self {
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            day_of_week: 0,
            day_of_month: 1,
            day_of_year: 1,
            month: 1,
            year: 1970,
        }
    }
}

pub fn get_time() -> RtcTime {
    let mut time1;
    let mut time2;
    while {
        time1 = get_time_maybe();
        time2 = get_time_maybe();
        time1 != time2
    } {}
    time1
}

fn get_time_maybe() -> RtcTime {
    unsafe {
        let rtc = RTC::ptr();
        let (time0, time1, time2) = ((*rtc).ctime0.read(), (*rtc).ctime1.read(), (*rtc).ctime2.read());
        RtcTime {
            seconds:  time0.seconds().bits(),
            minutes:  time0.minutes().bits(),
            hours:  time0.hours().bits(),
            day_of_week:  time0.dow().bits(),
            day_of_month:  time1.dom().bits(),
            month:  time1.month().bits(),
            year:  time1.year().bits(),
            day_of_year: time2.doy().bits(),
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Day {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
}

use self::Day::*;

impl Day {
    const VALUES: [Day; 7] = [Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday];
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Month {
    January, February, March, April, May, June, July, August, September, October, November, December
}

use self::Month::*;

impl Month {
    const VALUES: [Month; 12] = [January, February, March, April, May, June, July, August,
        September, October, November, December];
}

pub fn set_time(time: RtcTime) {
    unsafe {
        let rtc = RTC::ptr();
        // disable the RTC
        (*rtc).ccr.write(|w| w.clken().bit(false));

        (*rtc).sec.write(|w| w.seconds().bits(time.seconds));
        (*rtc).min.write(|w| w.minutes().bits(time.minutes));
        (*rtc).hrs.write(|w| w.hours().bits(time.hours));
        (*rtc).dow.write(|w| w.dow().bits(time.day_of_week));
        (*rtc).dom.write(|w| w.dom().bits(time.day_of_month));
        (*rtc).month.write(|w| w.month().bits(time.month));
        (*rtc).year.write(|w| w.year().bits(time.year));
        (*rtc).doy.write(|w| w.doy().bits(time.day_of_year));
        // enable the RTC
        (*rtc).ccr.write(|w| w.clken().bit(true));
    }
}

impl RtcTime {
    pub fn get_seconds(&self) -> u8 {
        self.seconds
    }
    pub fn set_seconds(&mut self, seconds: u8) -> bool {
        if seconds < 60 {
            self.seconds = seconds;
            true
        } else {
            false
        }
    }
    pub fn get_minutes(&self) -> u8 {
        self.minutes
    }
    pub fn set_minutes(&mut self, minutes: u8) -> bool {
        if minutes < 60 {
            self.minutes = minutes;
            true
        } else {
            false
        }
    }
    pub fn get_hours(&self) -> u8 {
        self.hours
    }
    pub fn set_hours(&mut self, hours: u8) -> bool {
        if hours < 24 {
            self.hours = hours;
            true
        } else {
            false
        }
    }
    pub fn get_day_of_week(&self) -> Day {
        Day::VALUES[self.day_of_week as usize]
    }
    pub fn set_day_of_week(&mut self, day: Day) {
        self.day_of_week = day as u8;
    }
    pub fn get_day_of_month(&self) -> u8 {
        self.day_of_month
    }
    pub fn set_day_of_month(&mut self, day: u8) -> bool {
        // I dare you to figure out how to break this
        if match self.month {
            2 => day < 29 ||
                (day < 30 && self.year % 4 == 0),
            4 | 6 | 9 | 11 => day < 31,
            _ => day < 32,
        } {
            self.day_of_month = day;
            true
        } else {
            false
        }
    }
    pub fn get_month(&self) -> Month {
        Month::VALUES[self.month as usize + 1]
    }
    pub fn set_month(&mut self, month: Month) {
        self.month = month as u8 + 1;
    }
    pub fn get_year(&self) -> u16 {
        self.year
    }
    pub fn set_year(&mut self, year: u16) {
        self.year = year;
    }
    pub fn get_day_of_year(&self) -> u16 {
        self.day_of_year
    }
    pub fn set_day_of_year(&mut self, day: u16) -> bool {
        if day < 366 ||
            self.year % 4 == 0 && day < 367
        {
            self.day_of_year = day;
            true
        } else {
            false
        }
    }
}
