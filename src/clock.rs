use lpc176x_5x::*;

unsafe fn sys_clock_disable() {
    // Disconnect PLL0
    (*SYSCON::ptr()).pll0con.write(|w| w.pllc0().bit(false));
    sys_clock_pll0_feed();
    // Disable PLL0
    (*SYSCON::ptr()).pll0con.write(|w| w.plle0().bit(false));
    sys_clock_pll0_feed();
    // Set clock to internal
    (*SYSCON::ptr()).clksrcsel.write(|w| w.clksrc().selects_the_internal());
    // Set m and n to 1
    (*SYSCON::ptr()).pll0cfg.write(|w| w.msel0().bits(0).nsel0().bits(0));
    sys_clock_pll0_feed();
    // Set CPU clock PLL divisor to 1
    (*SYSCON::ptr()).cclkcfg.write(|w| w.cclksel().bits(0));
}

#[inline]
unsafe fn sys_clock_pll0_feed() {
    // feed sequence to make PLL0 changes take effect
    (*SYSCON::ptr()).pll0feed.write(|w| w.pll0feed().bits(0xAA));
    (*SYSCON::ptr()).pll0feed.write(|w| w.pll0feed().bits(0x55));
}

pub unsafe fn sys_clock_configure() {
    // disable the clock for a minute
    sys_clock_disable();
//    let pll_input_khz = 4_000;
//    let mut cpu_clock_khz = 48_000;
//    // find good values to configure PLL0 with
//    let (m, n, d) = match sys_clock_configure_pll(cpu_clock_khz, pll_input_khz) {
//        Ok(t) => {
//            cpu_clock_khz = 24_000;
//            t
//        },
//        Err(t) => t,
//    };
//    (*SYSCON::ptr()).clksrcsel.write(|w| w.clksrc().selects_the_internal());
}

unsafe fn _sys_clock_configure_pll(desired_khz: u32, input_khz: u32)
    -> Result<(u32, u32, u32), (u32, u32, u32)>
{
    let mut last_viable = (0, 0, 0);
    for m in 511..=6 {
        for n in 0..32 {
            let min_khz = 275_000;
            let max_khz = 550_000;
            let khz = (2 * (m + 1) * input_khz) / (n + 1);

            if khz >= min_khz && khz <= max_khz {
                for cpudiv in 3..256 {
                    let cpu_clock_khz = khz / (cpudiv + 1);
                    let max_cpu_speed_khz = 100_000;
                    if cpu_clock_khz <= max_cpu_speed_khz {
                        last_viable = (m, n, cpudiv);
                        if cpu_clock_khz == desired_khz {
                            return Ok(last_viable);
                        }
                    }
                }
            }
        }
    }
    Err(last_viable)
}

pub fn get_cpu_clock() -> u32 {
    unsafe {
        let pll0stat = (*SYSCON::ptr()).pll0stat.read();
        if pll0stat.plle0_stat().bit() && pll0stat.pllc0_stat().bit() {
            let clksrc = (*SYSCON::ptr()).clksrcsel.read().clksrc();
            if clksrc.is_selects_the_main_osc() {
                (24_000_000_u64 * (pll0stat.msel0().bits() + 1) as u64
                    / (pll0stat.nsel0().bits() + 1) as u64
                    / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u64) as u32
            } else if clksrc.is_selects_the_rtc_osci() {
                (65_536_u64 * (pll0stat.msel0().bits() + 1) as u64
                    / (pll0stat.nsel0().bits() + 1) as u64
                    / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u64) as u32
            } else {
                (8_000_000_u64 * (pll0stat.msel0().bits() + 1) as u64
                    / (pll0stat.nsel0().bits() + 1) as u64
                    / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u64) as u32
            }
        } else {
            let clksrc = (*SYSCON::ptr()).clksrcsel.read().clksrc();
            if clksrc.is_selects_the_main_osc() {
                12_000_000 / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u32
            } else if clksrc.is_selects_the_rtc_osci() {
                32_768 / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u32
            } else {
                4_000_000 / ((*SYSCON::ptr()).cclkcfg.read().cclksel().bits() + 1) as u32
            }
        }
    }
}
