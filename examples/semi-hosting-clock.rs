#![no_main]
#![no_std]

extern crate stm32f103xx_rtc as rtc;
extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
#[macro_use]
extern crate stm32f103xx as device;
extern crate stm32f103xx_hal as hal;
extern crate cortex_m_semihosting as sh;
extern crate heapless;

use rt::ExceptionFrame;
use core::fmt::Write;
use hal::prelude::*;

entry!(main);

static mut RTC_DEVICE: Option<rtc::Rtc> = None;

fn main() -> ! {
    let mut dp = device::Peripherals::take().unwrap();
    let mut cp = device::CorePeripherals::take().unwrap();
    let mut rcc = dp.RCC.constrain();

    let mut rtc = rtc::Rtc::new(dp.RTC, &mut rcc.apb1, &mut dp.PWR);
    if rtc.get_cnt() < 100 {
        rtc.set_cnt(1534199480 + 2 * 60 * 60);
    }
    unsafe {
        RTC_DEVICE = Some(rtc);
        RTC_DEVICE.as_mut().unwrap().enable_second_interrupt(&mut cp.NVIC);
    }

    loop {
        cortex_m::asm::wfi();
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

interrupt!(RTC, rtc);

fn rtc() {
    let mut hstdout = sh::hio::hstdout().unwrap();
    let rtc = unsafe { RTC_DEVICE.as_mut().unwrap() };
    let mut s = heapless::String::<heapless::consts::U32>::new();
    writeln!(s, "{}", rtc::datetime::DateTime::new(rtc.get_cnt())).unwrap();
    hstdout.write_str(&s).unwrap();
    rtc.clear_second_interrupt();
}
