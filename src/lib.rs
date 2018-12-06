#![no_std]

extern crate stm32f103xx_hal as hal;

use crate::hal::device as device;

pub struct Rtc {
    rtc: device::RTC
}
pub struct RtcCommit<'a>(&'a mut Rtc);
impl<'a> Drop for RtcCommit<'a> {
    fn drop(&mut self) { self.0.commit(); }
}
impl Rtc {
    pub fn new(
        rtc: device::RTC,
        _apb1: &mut hal::rcc::APB1,
        pwr: &mut device::PWR,
    ) -> Rtc {
        let rcc = unsafe { &*device::RCC::ptr() };
        let mut rtc = Rtc { rtc };

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

        // setting freq
        let freq = 32768;
        let prl = freq - 1;
        assert!(prl < 1 << 20);
        rtc.modify(|s| {
            s.rtc.prlh.write(|w| unsafe { w.bits(prl >> 16) });
            s.rtc.prll.write(|w| unsafe { w.bits(prl as u16 as u32) });
        });

        rtc.sync();
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
        self.modify(|s| {
            s.rtc.cntl.write(|w| unsafe { w.cntl().bits(cnt as u16) });
            s.rtc.cnth.write(|w| unsafe { w.cnth().bits((cnt >> 16) as u16) });
        })
    }
    pub fn enable_second_interrupt(&mut self, nvic: &mut device::NVIC) {
        self.rtc.crh.write(|w| w.secie().set_bit());
        nvic.enable(device::Interrupt::RTC);
    }
    pub fn clear_second_interrupt(&mut self) {
        self.rtc.crl.write(|w| w.secf().clear_bit());
    }
    fn modify<F: FnOnce(&mut Self)>(&mut self, f: F) -> RtcCommit {
        self.sync();
        self.rtc.crl.modify(|_, w| w.cnf().set_bit());
        f(self);
        RtcCommit(self)
    }
    fn commit(&mut self) {
        self.rtc.crl.modify(|_, w| w.cnf().clear_bit());
        self.sync();
    }
}
