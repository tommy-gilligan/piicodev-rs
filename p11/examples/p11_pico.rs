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

    use embedded_hal::delay::DelayUs;
    use p11::P11;
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

        let delay_1 = MyDelay(delay);
        let mut p11 = P11::new(i2c, 0x76, delay_1).unwrap();

        let temperature_and_pressure = p11.read_temperature_and_pressure(None).unwrap();
        info!(
            "{:?} {:?}",
            temperature_and_pressure.0.as_celsius(),
            temperature_and_pressure.1.as_hectopascals()
        );
        info!("light on!");
        led_pin.set_high().unwrap();
        loop {}
    }
}
