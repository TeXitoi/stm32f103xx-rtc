#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate cortex_m_semihosting as sh;
extern crate heapless;

use rt::ExceptionFrame;
use core::fmt::Write;
use hal::stm32f103xx;
use hal::prelude::*;

entry!(main);

pub struct Rtc {
    rtc: stm32f103xx::RTC
}
pub struct RtcCommit<'a>(&'a mut Rtc);
impl<'a> Drop for RtcCommit<'a> {
    fn drop(&mut self) { self.0.commit(); }
}
impl Rtc {
    pub fn new(
        rtc: stm32f103xx::RTC,
        _apb1: &mut hal::rcc::APB1,
        pwr: &mut stm32f103xx::PWR,
    ) -> Rtc {
        let rcc = unsafe { &*hal::stm32f103xx::RCC::ptr() };
        let rtc = Rtc { rtc };
        if rcc.apb1enr.read().bkpen().is_disabled() {
            // Power on
            rcc.apb1enr.modify(|_, w| w.pwren().enabled());
            rcc.apb1enr.modify(|_, w| w.bkpen().enabled());
            pwr.cr.modify(|_, w| w.dbp().set_bit());

            // Selecting Low Speed External clock
            rcc.bdcr.modify(|_, w| w.lsebyp().clear_bit());
            rcc.bdcr.modify(|_, w| w.lseon().set_bit());
            while rcc.bdcr.read().lserdy().bit_is_clear() {}
            rcc.bdcr.modify(|_, w| w.rtcsel().lse());

            // enable RTC
            rcc.bdcr.modify(|_, w| w.rtcen().set_bit());

            rtc.sync();
        }
        rtc
    }
    pub fn sync(&self) {
        while self.rtc.crl.read().rsf().bit_is_clear() {}
        while self.rtc.crl.read().rtoff().bit_is_clear() {}
    }
    pub fn get_cnt(&self) -> u32 {
        self.rtc.cnth.read().bits() << 16 | self.rtc.cntl.read().bits()
    }
    pub fn set_cnt(&mut self, cnt: u32) -> RtcCommit {
        self.sync();
        self.rtc.crl.modify(|_, w| w.cnf().set_bit());
        self.rtc.cntl.write(|w| unsafe { w.cntl().bits(cnt as u16) });
        self.rtc.cnth.write(|w| unsafe { w.cnth().bits((cnt >> 16) as u16) });
        RtcCommit(self)
    }
    fn commit(&mut self) {
        self.rtc.crl.modify(|_, w| w.cnf().clear_bit());
        self.sync();
    }
}

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
        use DayOfWeek::*;
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
impl core::fmt::Display for DayOfWeek {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
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
impl core::fmt::Display for DateTime {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
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

fn main() -> ! {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let mut dp = hal::stm32f103xx::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();
    let mut rtc = Rtc::new(dp.RTC, &mut rcc.apb1, &mut dp.PWR);
    if rtc.get_cnt() < 100 {
        rtc.set_cnt(1534026785 + 2 * 60 * 60);
    }

    loop {
        let mut s = heapless::String::<heapless::consts::U32>::new();
        writeln!(s, "{}", DateTime::new(rtc.get_cnt())).unwrap();
        hstdout.write_str(&s).unwrap();
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
