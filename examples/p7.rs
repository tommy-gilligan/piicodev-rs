#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
mod other {
    extern crate std;
    use std::println;
    #[no_mangle]
    pub extern "C" fn main() {
        loop {
            println!("unsupported target");
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod arm {
    use defmt::*;
    use defmt_rtt as _;
    use fugit::RateExtU32;
    use panic_probe as _;
    use piicodev::DriverUsingDelay;
    use rp2040_hal::{
        clocks::{init_clocks_and_plls, Clock},
        entry,
        i2c::I2C,
        pac,
        sio::Sio,
        watchdog::Watchdog,
    };

    use core::cell::RefCell;
    use piicodev::p7::P7;

    use embedded_hal::delay::DelayUs;
    #[derive(Debug, PartialEq)]
    struct MyDelayError;
    struct MyDelay<'a>(&'a RefCell<cortex_m::delay::Delay>);
    impl DelayUs for MyDelay<'_> {
        fn delay_us(&mut self, s: u32) -> () {
            self.0.borrow_mut().delay_us(s);
        }
    }

    #[link_section = ".boot2"]
    #[used]
    pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

    #[entry]
    fn main() -> ! {
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        let external_xtal_freq_hz = 12_000_000u32;
        let clocks = init_clocks_and_plls(
            external_xtal_freq_hz,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let pins = rp2040_hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio8.into_function(), // sda
            pins.gpio9.into_function(), // scl
            400.kHz(),
            &mut pac.RESETS,
            100_000_000.Hz(),
        );

        let delay = &RefCell::new(cortex_m::delay::Delay::new(
            core.SYST,
            clocks.system_clock.freq().to_Hz(),
        ));
        let mut p7 = P7::new(i2c, 0x29, MyDelay(delay)).unwrap().init().unwrap();
        let mut delay = delay.borrow_mut();

        loop {
            // read the distance in millimetres
            let dist = p7.read().unwrap();
            // convert the number to a string and print
            println!("{} mm", dist);
            delay.delay_us(100_000);
        }
    }
}
