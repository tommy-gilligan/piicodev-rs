#![no_std]
#![no_main]

mod linux {
    extern crate std;
    use linux_embedded_hal::I2cdev;
    use p13::P13;
    use smart_leds_trait::RGB;
    use std::env;
    use std::fs;
    use std::path;

    use crate::linux::path::PathBuf;

    fn path_for_bus(needle: u8) -> Option<path::PathBuf> {
        for dir_entry in fs::read_dir("/sys/class/i2c-dev").unwrap() {
            let dir_entry = dir_entry.unwrap();
            let file_name = dir_entry.file_name();
            let file_str = file_name.to_str().unwrap();
            let number: u8 = file_str.strip_prefix("i2c-").unwrap().parse().unwrap();
            let path = PathBuf::from("/dev");

            let path = path.join(dir_entry.path().file_name().unwrap());
            let _metadata = path.metadata().unwrap().file_type();
            if number == needle {
                return Some(path);
            }
        }
        None
    }

    #[no_mangle]
    pub extern "C" fn main() {
        // handles only as decimal but should accept hexadecimal
        let mut args = env::args().skip(1);
        let i2c_bus: u8 = args.next().unwrap().parse().unwrap();
        let i2c_address: u8 = args
            .next()
            .unwrap()
            .parse()
            .expect("Error: Chip address is not a number!");
        if !(0x03..=0x77).contains(&i2c_address) {
            panic!("Error: Chip address out of range (0x03-0x77)!");
        }

        let i2c = I2cdev::new(path_for_bus(i2c_bus).unwrap()).unwrap();
        let mut p13 = P13::new(i2c, i2c_address);

        let data: [RGB<u8>; 3] = [
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0xff,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0xff,
            },
        ];
        use smart_leds_trait::SmartLedsWrite;
        p13.write(data.iter().cloned()).unwrap();
    }
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
    use embedded_hal::digital::OutputPin;
    use fugit::RateExtU32;
    use panic_probe as _;
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

    use p13::P13;
    use smart_leds_trait::RGB;

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

        let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

        let pins = rp_pico::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );

        let mut led_pin = pins.led.into_push_pull_output();

        let i2c = I2C::i2c0(
            pac.I2C0,
            pins.gpio8.into_mode(), // sda
            pins.gpio9.into_mode(), // scl
            400.kHz(),
            &mut pac.RESETS,
            100_000_000.Hz(),
        );

        info!("light off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
        let mut p13 = P13::new(i2c, 0x0B);

        let data: [RGB<u8>; 3] = [
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0xff,
                b: 0x00,
            },
            RGB {
                r: 0xff,
                g: 0x00,
                b: 0xff,
            },
        ];
        use smart_leds_trait::SmartLedsWrite;
        p13.write(data.iter().cloned()).unwrap();

        info!("light on!");
        led_pin.set_high().unwrap();
        loop {}
    }
}
