#![no_std]
#![no_main]

#[cfg(target_os = "linux")]
mod linux {
    extern crate std;
    use embedded_hal::delay::DelayUs;
    use fugit::ExtU32;
    use fugit::HertzU32;
    use linux_embedded_hal::{Delay, I2cdev};
    use p18::P18;
    use std::env;
    use std::fs;
    use std::path;
    use std::println;

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
        let mut p18 = P18::new(i2c, i2c_address);
        let mut delay = Delay;

        let notes: [u8; 12] = [64, 66, 71, 73, 74, 66, 64, 73, 71, 66, 74, 73];
        loop {
            for note in notes {
                let frequency: HertzU32 = HertzU32::from_raw(
                    (440_f64 * 2_f64.powf((note as f64 - 69_f64) / 12_f64)) as u32,
                );
                p18.tone(frequency, 25.millis()).unwrap();
                println!("{}", frequency);
                delay.delay_ms(50);
            }
            delay.delay_ms(500);
        }
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
    use fugit::{ExtU32, RateExtU32};
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

    use p18::P18;

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
        let mut p18 = P18::new(i2c, 0x5C);

        loop {
            p18.tone(540u32.Hz(), 1000u32.millis()).unwrap();
            info!("{:?}", p18.read_status().unwrap());
            info!("light on!");
            led_pin.set_high().unwrap();
            delay.delay_ms(2000);
            info!("light off!");
            led_pin.set_low().unwrap();
            delay.delay_ms(2000);
        }
    }
}
