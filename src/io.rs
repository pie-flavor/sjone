use core::fmt::{Write, Result as FmtResult};

use lpc176x_5x::*;

pub(crate) unsafe fn uart0_init() {
    let divider = (crate::clock::get_cpu_clock() as f64 / (16. * 38400.) + 0.5) as u16;
    let [lsb, msb] = divider.to_le_bytes();
    // enable power to UART0
    (*SYSCON::ptr()).pconp.write(|w| w.pcuart0().bit(true));
    // enable peripheral clock access to UART0
    (*SYSCON::ptr()).pclksel0.write(|w| w.pclk_uart0().cclk());
    // enable pins access to UART0
    (*PINCONNECT::ptr()).pinsel0.write(|w| w.p0_2().txd0().p0_3().rxd0());
    // enable divisor latch access
    (*UART0::ptr()).lcr.write(|w| w.dlab().enable_access_to_div());
    // set divisor latch value
    (*UART0::ptr()).dlm.dlm.write(|w| w.dlmsb().bits(msb));
    (*UART0::ptr()).dll.dll.write(|w| w.dllsb().bits(lsb));
    // disable divisor latch value
    (*UART0::ptr()).lcr.write(|w| w.dlab().disable_access_to_di());
}

pub struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, str: &str) -> FmtResult {
        for byte in str.as_bytes() {
            Stdout.write_char(*byte);
        }
        Ok(())
    }
}

impl Stdout {
    pub fn write_char(&self, c: u8) {
        unsafe {
            (*UART0::ptr()).dll.thr.write(|w| w.thr().bits(c));
            while (*UART0::ptr()).lsr.read().temt().is_validdata() {}
        }
    }
}

pub struct Stdin;

impl Stdin {
    pub fn read_char(&self) -> u8 {
        unsafe {
            while !(*UART0::ptr()).lsr.read().rdr().bit() {}
            (*UART0::ptr()).dll.rbr.read().rbr().bits()
        }
    }
}