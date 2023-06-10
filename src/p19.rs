//! # Unofficial Rust Driver for PiicoDev Real Time Clock
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-Real-Time-Clock-RV3028
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-RV3028-MicroPython-Module
//! [Official Product Site]: https://piico.dev/p19
//! [Datasheet]: https://www.microcrystal.com/fileadmin/Media/Products/RTC/App.Manual/RV-3028-C7_App-Manual.pdf
use embedded_hal::i2c::I2c;

const REG_STATUS: u8 = 0x0E;
const REG_UNIX: u8 = 0x1B;
const REG_ID: u8 = 0x28;
const REG_EEPROM_BACKUP: u8 = 0x37;

use num_enum::IntoPrimitive;
#[derive(IntoPrimitive)]
#[repr(u8)]
pub enum TrickleResistance {
    Resistance3kΩ = 0,
    Resistance5kΩ = 1,
    Resistance9kΩ = 2,
    Resistance15kΩ = 3,
}

pub struct P19<I2C> {
    i2c: I2C,
    address: u8,
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C> for P19<I2C> {
    fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<I2C: I2c> P19<I2C> {
    pub fn init(mut self) -> Result<Self, I2C::Error> {
        self.whoami()?;
        self.set_battery_switchover(true)?;
        self.config_trickle_charger(TrickleResistance::Resistance3kΩ)?;
        self.set_trickle_charger(true)?;
        Ok(self)
    }

    pub fn set_battery_switchover(&mut self, switchover_enabled: bool) -> Result<(), I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_EEPROM_BACKUP], &mut data)?;
        let new_ee_backup = if switchover_enabled {
            (data[0] & 0b1111_0011) | 0b0000_0100
        } else {
            data[0] & 0b1111_0011
        };
        self.i2c
            .write(self.address, &[REG_EEPROM_BACKUP, new_ee_backup])?;

        Ok(())
    }

    pub fn config_trickle_charger(
        &mut self,
        trickle_resistance: TrickleResistance,
    ) -> Result<(), I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_EEPROM_BACKUP], &mut data)?;
        self.i2c.write(
            self.address,
            &[
                REG_EEPROM_BACKUP,
                ((data[0] | 0x80) & 0b1111_1100)
                    | <TrickleResistance as core::convert::Into<u8>>::into(trickle_resistance),
            ],
        )?;

        Ok(())
    }

    pub fn set_trickle_charger(&mut self, tricker_charger: bool) -> Result<(), I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_EEPROM_BACKUP], &mut data)?;
        let new_ee_backup = if tricker_charger {
            data[0] | 0b0010_0000
        } else {
            data[0] & 0b1101_1111
        };
        self.i2c
            .write(self.address, &[REG_EEPROM_BACKUP, new_ee_backup])?;

        Ok(())
    }

    pub fn whoami(&mut self) -> Result<u8, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c.write_read(self.address, &[REG_ID], &mut data)?;
        Ok(data[0])
    }

    pub fn get_unix_time(&mut self) -> Result<u32, I2C::Error> {
        let mut data: [u8; 4] = [0, 0, 0, 0];
        self.i2c.write_read(self.address, &[REG_UNIX], &mut data)?;
        Ok(u32::from_le_bytes(data))
    }

    pub fn set_unix_time(&mut self, unix_time: u32) -> Result<(), I2C::Error> {
        let mut data: [u8; 5] = [REG_UNIX; 5];
        for (i, b) in u32::to_le_bytes(unix_time).into_iter().enumerate() {
            data[i + 1] = b;
        }
        self.i2c.write(self.address, &data)?;

        Ok(())
    }

    pub fn check_alarm(&mut self) -> Result<bool, I2C::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address, &[REG_STATUS], &mut data)?;
        if (data[0] & 0b0000_0100) == 0 {
            Ok(false)
        } else {
            self.i2c
                .write(self.address, &[REG_STATUS, data[0] & 0b1111_1011])?;
            Ok(true)
        }
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    use crate::Driver;
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p19::{TrickleResistance, P19};

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x28], vec![201]),
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x19]),
            I2cTransaction::write(0x52, vec![0x37, 0x15]),
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x00]),
            I2cTransaction::write(0x52, vec![0x37, 0x80]),
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x00]),
            I2cTransaction::write(0x52, vec![0x37, 0x20]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P19::new(i2c, 0x52).init().unwrap();

        i2c_clone.done();
    }

    #[test]
    pub fn set_battery_switchover_true() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x19]),
            I2cTransaction::write(0x52, vec![0x37, 0x15]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.set_battery_switchover(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_battery_switchover_false() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x19]),
            I2cTransaction::write(0x52, vec![0x37, 0x11]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.set_battery_switchover(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn config_trickle_charger() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x00]),
            I2cTransaction::write(0x52, vec![0x37, 0x83]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(
            p19.config_trickle_charger(TrickleResistance::Resistance15kΩ),
            Ok(())
        );
        i2c_clone.done();
    }

    #[test]
    pub fn set_trickle_charger_true() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x00]),
            I2cTransaction::write(0x52, vec![0x37, 0x20]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.set_trickle_charger(true), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn set_trickle_charger_false() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x37], vec![0x20]),
            I2cTransaction::write(0x52, vec![0x37, 0x00]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.set_trickle_charger(false), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn whoami() {
        let expectations = [I2cTransaction::write_read(0x52, vec![0x28], vec![201])];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.whoami(), Ok(201));
        i2c_clone.done();
    }

    #[test]
    pub fn get_unix_time() {
        let expectations = [I2cTransaction::write_read(
            0x52,
            vec![0x1B],
            vec![0x00, 0x63, 0x58, 0x64],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.get_unix_time(), Ok(1_683_514_112));
        i2c_clone.done();
    }

    #[test]
    pub fn set_unix_time() {
        let expectations = [I2cTransaction::write(
            0x52,
            vec![0x1B, 0x00, 0x63, 0x58, 0x64],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.set_unix_time(1_683_514_112), Ok(()));
        i2c_clone.done();
    }

    #[test]
    pub fn check_alarm_true() {
        let expectations = [
            I2cTransaction::write_read(0x52, vec![0x0E], vec![0b0011_0100]),
            I2cTransaction::write(0x52, vec![0x0E, 0b0011_0000]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.check_alarm(), Ok(true));
        i2c_clone.done();
    }

    #[test]
    pub fn check_alarm_false() {
        let expectations = [I2cTransaction::write_read(
            0x52,
            vec![0x0E],
            vec![0b0011_0000],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p19 = P19 { i2c, address: 0x52 };

        assert_eq!(p19.check_alarm(), Ok(false));
        i2c_clone.done();
    }
}
