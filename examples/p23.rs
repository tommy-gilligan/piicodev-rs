#![no_std]
#![no_main]

#[cfg(target_os = "linux")]
mod linux {
    extern crate std;

    #[no_mangle]
    pub const extern "C" fn main() {}
}

#[cfg(not(any(target_os = "linux", target_os = "none")))]
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
    use embedded_hal::delay::DelayUs;
    use fugit::RateExtU32;
    use panic_probe as _;
    use piicodev::p23::{AirQuality, P23};
    use rp_pico::{
        entry,
        hal::{
            clocks::{init_clocks_and_plls, Clock},
            i2c::I2C,
            pac,
            sio::Sio,
            watchdog::Watchdog,
        },
    };

    #[derive(Debug, PartialEq)]
    struct MyDelayError;
    struct MyDelay(cortex_m::delay::Delay);
    impl DelayUs for MyDelay {
        fn delay_us(&mut self, s: u32) -> () {
            self.0.delay_us(s);
        }
    }

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

        let pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio16.into_mode(), // sda
            pins.gpio17.into_mode(), // scl
            400.kHz(),
            &mut pac.RESETS,
            100_000_000.Hz(),
        );

        let mut p23 = P23::new(i2c, 0x53).unwrap();
        loop {
            if p23.data_ready().unwrap() {
                let read = p23.read().unwrap();
                println!("{:?}", read.aqi);
                println!("{:?}", read.tvoc);
                println!("{:?}", read.eco2);
            }
        }
    }
}
