#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate cortex_m_semihosting as sh;

use rt::ExceptionFrame;
use core::fmt::Write;

entry!(main);

fn main() -> ! {
    let mut hstdout = sh::hio::hstdout().unwrap();

    let raw_rcc = unsafe { &*hal::stm32f103xx::RCC::ptr() };
    let raw_pwr = unsafe { &*hal::stm32f103xx::PWR::ptr() };
    let raw_rtc = unsafe { &*hal::stm32f103xx::RTC::ptr() };

    raw_rcc.apb1enr.modify(|_, w| w.pwren().enabled());
    raw_rcc.apb1enr.modify(|_, w| w.bkpen().enabled());

    raw_pwr.cr.modify(|_, w| w.dbp().set_bit());

    raw_rcc.bdcr.modify(|_, w| w.lsebyp().clear_bit());
    raw_rcc.bdcr.modify(|_, w| w.lseon().set_bit());

    while raw_rcc.bdcr.read().lserdy().bit_is_clear() {}
    writeln!(hstdout, "LSE ready").unwrap();

    raw_rcc.bdcr.modify(|_, w| w.rtcen().set_bit());

    raw_rcc.bdcr.modify(|_, w| w.rtcsel().lse());

    while raw_rtc.crl.read().rsf().bit_is_clear() {}
    writeln!(hstdout, "RTC sync").unwrap();

    while raw_rtc.crl.read().rtoff().bit_is_clear() {}
    writeln!(hstdout, "RTC done").unwrap();

    loop {
        let clock = raw_rtc.cnth.read().bits() << 16 | raw_rtc.cntl.read().bits();
        writeln!(hstdout, "rtc = {}", clock).unwrap();
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
