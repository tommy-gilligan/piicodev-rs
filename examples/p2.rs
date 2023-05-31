#![no_std]
#![no_main]

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

#[cfg(target_os = "linux")]
mod linux {
    extern crate std;
    use embedded_hal::{delay::DelayUs, i2c::I2c};
    use linux_embedded_hal::{Delay, I2cdev};
    use std::env;
    use std::fs;
    use std::path;

    use crate::linux::path::PathBuf;

    fn path_for_bus(needle: u8) -> Option<path::PathBuf> {
        for dir_entry in fs::read_dir("/sys/class/i2c-dev").unwrap() {
            let dir_entry_e = dir_entry.unwrap();
            let file_name = dir_entry_e.file_name();
            let file_str = file_name.to_str().unwrap();
            let number: u8 = file_str.strip_prefix("i2c-").unwrap().parse().unwrap();
            let path = PathBuf::from("/dev");

            let path_e = path.join(dir_entry_e.path().file_name().unwrap());
            let _metadata = path_e.metadata().unwrap().file_type();
            if number == needle {
                return Some(path_e);
            }
        }
        None
    }

    fn arguments() -> (u8, u8) {
        // handles only as decimal but should accept hexadecimal
        let mut args = env::args().skip(1);
        let i2c_bus: u8 = args.next().unwrap().parse().unwrap();
        let i2c_address: u8 = args
            .next()
            .unwrap()
            .parse()
            .expect("Error: Chip address is not a number!");
        assert!(
            (0x03..=0x77).contains(&i2c_address),
            "Error: Chip address out of range (0x03-0x77)!"
        );

        (i2c_bus, i2c_address)
    }

    #[no_mangle]
    pub extern "C" fn main() {
        let (i2c_bus, i2c_address) = arguments();

        example(
            &I2cdev::new(path_for_bus(i2c_bus).unwrap()).unwrap(),
            i2c_address,
            Delay,
        )
        .unwrap();
    }

    fn example<I2C: I2c, DELAY: DelayUs>(
        _i2c: &I2C,
        _i2c_address: u8,
        mut delay: DELAY,
    ) -> Result<(), I2C::Error> {
        loop {
            delay.delay_ms(500);
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod arm {
    use defmt::*;
    use defmt_rtt as _;
    use embedded_hal::delay::DelayUs;
    use embedded_hal::i2c::I2c;
    use fugit::RateExtU32;
    use panic_probe as _;
    use piicodev::p2::P2;
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

        let mut i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio16.into_mode(), // sda
            pins.gpio17.into_mode(), // scl
            400.kHz(),
            &mut pac.RESETS,
            100_000_000.Hz(),
        );

        let mut p2 = P2::new(i2c, 0x77).unwrap();
        loop {
            println!("{:?}", p2.centi_celsius().unwrap());
        }
    }
}
