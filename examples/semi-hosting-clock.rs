#![no_main]
#![no_std]

extern crate stm32f103xx_rtc as rtc;
extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32f103xx_hal as hal;
extern crate cortex_m_semihosting as sh;
extern crate heapless;

use core::fmt::Write;
use hal::prelude::*;
use rt::entry;
use hal::device::interrupt;

static mut RTC_DEVICE: Option<rtc::Rtc> = None;

#[entry]
fn main() -> ! {
    let mut dp = hal::device::Peripherals::take().unwrap();
    let mut cp = hal::device::CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    let mut rtc = rtc::Rtc::new(dp.RTC, &mut rcc.apb1, &mut dp.PWR);
    if rtc.get_cnt() < 100 {
        rtc.set_cnt(4242);
    }
    unsafe {
        RTC_DEVICE = Some(rtc);
        RTC_DEVICE.as_mut().unwrap().enable_second_interrupt(&mut cp.NVIC);
    }

    loop {
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn RTC() {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let rtc = unsafe { RTC_DEVICE.as_mut().unwrap() };
    let mut s = heapless::String::<heapless::consts::U32>::new();
    writeln!(s, "{}", rtc.get_cnt()).unwrap();
    hstdout.write_str(&s).unwrap();
    rtc.clear_second_interrupt();
}
