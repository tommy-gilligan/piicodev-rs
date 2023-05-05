#![no_std]
#![no_main]

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
mod not_arm {
    extern crate std;
    #[no_mangle]
    pub extern "C" fn main() {
        std::println!("unsupported target");
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

    use p30::P30;

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

        let mut p30 = P30::new(i2c, 0x35).unwrap();
        p30.set_led(false).unwrap();
        p30.set_period(10_000).unwrap();

        loop {
            if p30.new_sample_available().unwrap() {
                println!(
                    "{}ms, {}cm",
                    p30.round_trip_time().unwrap(),
                    p30.distance().unwrap().as_centimetres(),
                )
            }
            delay.delay_ms(1000);
        }
    }
}
