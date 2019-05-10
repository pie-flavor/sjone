#![no_std]
#![feature(global_asm, asm, naked_functions)]
#![allow(dead_code)]

pub mod devices;
mod clock;
pub mod rtc;
pub mod io;

use core::panic::PanicInfo;
use core::fmt::{Debug, Formatter, Write, Result as FmtResult};

pub use lpc176x_5x::*;

use lpc176x_5x::interrupt;
use cortex_m_rt_macros::interrupt;

unsafe fn setup() -> ! {
    rtc::rtc_init();
    clock::sys_clock_configure();
    cfg_flash_acceleration(clock::get_cpu_clock());
    io::uart0_init();
//    enable_watchdog();
//    prog_start();
    write!(&mut crate::io::Stdout, "Successfully started").ok();
    loop {}
}

unsafe fn cfg_flash_acceleration(clock: u32) {
    let mhz = clock / 1_000_000;
    match mhz {
        0..=20 => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._1clk()),
        21..=40 => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._2clk()),
        41..=60 => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._3clk()),
        61..=80 => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._4clk()),
        81..=100 => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._5clk()),
        _ => (*SYSCON::ptr()).flashcfg.write(|w| w.flashtim()._6clk()),
    }
}

unsafe fn enable_watchdog() {
    let watchdog_timeout = 3_000_000; //microseconds
    // set the timeout
    (*WDT::ptr()).tc.write(|w| w.count().bits(watchdog_timeout));
    // enable the watchdog and the watchdog reset
    (*WDT::ptr()).mod_.write(|w| w.wden().bit(true).wdreset().bit(true));
    feed_watchdog();
    setup_watchdog_feeder();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    writeln!(crate::io::Stdout, "{}", info).ok();
    loop {}
}

//extern "Rust" {
//    fn prog_start();
//}

pub fn feed_watchdog() {
    unsafe {
        (*WDT::ptr()).feed.write(|w| w.feed().bits(0xAA));
        (*WDT::ptr()).feed.write(|w| w.feed().bits(0x55));
    }
}

unsafe fn setup_watchdog_feeder() {
    (*RTC::ptr()).ciir.write(|w| w.imsec().set_bit());
}

#[interrupt]
#[allow(non_snake_case)]
unsafe fn RTC() {
    if (*RTC::ptr()).ilr.read().rtccif().bit_is_set() {
        (*RTC::ptr()).ilr.write(|w| w.rtccif().set_bit());
        feed_watchdog();
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExceptionFrame {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
}

impl Debug for ExceptionFrame {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        struct Hex(u32);
        impl Debug for Hex {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                write!(f, "0x{:08x}", self.0)
            }
        }
        f.debug_struct("ExceptionFrame")
            .field("r0", &Hex(self.r0))
            .field("r1", &Hex(self.r1))
            .field("r2", &Hex(self.r2))
            .field("r3", &Hex(self.r3))
            .field("r12", &Hex(self.r12))
            .field("lr", &Hex(self.lr))
            .field("pc", &Hex(self.pc))
            .field("xpsr", &Hex(self.xpsr))
            .finish()
    }
}

#[inline]
pub fn heap_start() -> *mut u32 {
    extern "C" {
        static mut __sheap: u32;
    }
    unsafe { &mut __sheap }
}

#[doc(hidden)]
#[link_section = ".vector_table.reset_vector"]
#[no_mangle]
pub static __RESET_VECTOR: unsafe extern "C" fn () -> ! = Reset;

#[doc(hidden)]
#[allow(non_snake_case)]
#[naked]
pub unsafe extern "C" fn Reset() -> ! {
    extern "C" {
        static mut __sbss: u32;
        static mut __ebss: u32;
        static mut __sdata: u32;
        static mut __edata: u32;
        static __sidata: u32;
    }

//    extern "Rust" {
//        fn prog_start() -> !;
//    }
    setup_stack();
    atomic::fence(atomic::Ordering::SeqCst);
    r0::init_data(&mut __sdata, &mut __edata, &__sidata);
    r0::zero_bss(&mut __sbss, &mut __ebss);
    setup()
}

#[allow(unused_variables, non_snake_case)]
#[doc(hidden)]
#[link_section = ".HardFault.default"]
#[no_mangle]
pub unsafe extern "C" fn HardFault_(ef: &ExceptionFrame) -> ! {
    loop {
        atomic::fence(atomic::Ordering::SeqCst);
    }
}

#[doc(hidden)]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn DefaultHandler_() -> ! {
    loop {
        atomic::fence(atomic::Ordering::SeqCst);
    }
}

#[doc(hidden)]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn DefaultPreInit() {}

#[doc(hidden)]
pub enum Exception {
    NonMaskableInt,
    MemoryManagement,
    BusFault,
    UsageFault,
    SVCall,
    DebugMonitor,
    PendSV,
    SysTick,
}

extern "C" {
    fn NonMaskableInt();
    fn HardFaultTrampoline();
    fn MemoryManagement();
    fn BusFault();
    fn UsageFault();
    fn SVCall();
    fn DebugMonitor();
    fn PendSV();
    fn SysTick();
}

pub union Vector {
    handler: unsafe extern "C" fn (),
    reserved: usize,
}

#[doc(hidden)]
#[link_section = ".vector_table.exceptions"]
#[no_mangle]
pub static __EXCEPTIONS: [Vector; 14] = [
    Vector { handler: NonMaskableInt },
    Vector { handler: HardFaultTrampoline },
    Vector { handler: MemoryManagement },
    Vector { handler: BusFault },
    Vector { handler: UsageFault },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: SVCall },
    Vector { handler: DebugMonitor },
    Vector { reserved: 0 },
    Vector { handler: PendSV },
    Vector { handler: SysTick },
];

#[naked]
unsafe fn setup_stack() {
    extern "C" {
        static __stack_top: u32;
    }
    asm!("MSR psp, $0\nBX lr" :: "r" (&__stack_top) :: "volatile");
    asm!("MSR msp, $0\nBX lr" :: "r" (&__stack_top) :: "volatile");
}

global_asm!(include_str!("asm.s"));
