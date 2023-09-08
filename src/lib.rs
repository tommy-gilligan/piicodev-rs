#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetAddressError<E> {
    I2cError(E),
    ArgumentError,
}
use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;
pub trait Atmel<I2C: I2c> {
    fn get_led(&mut self) -> Result<bool, I2C::Error>;
    fn set_led(&mut self, on: bool) -> Result<(), I2C::Error>;
    fn firmware(&mut self) -> Result<(u8, u8), I2C::Error>;
    fn set_address(&mut self, new_address: u8) -> Result<(), SetAddressError<I2C::Error>>;
}

pub trait WhoAmI<I2C: I2c, T: core::cmp::Eq> {
    const EXPECTED_WHOAMI: T;

    fn whoami(&mut self) -> Result<T, I2C::Error>;
}

#[derive(Debug)]
pub struct OutOfRange;
pub trait Driver<I2C: I2c, T> {
    fn address_check(address: u8) -> Result<(), OutOfRange> {
        if (0x08..=0x77).contains(&address) {
            Ok(())
        } else {
            Err(OutOfRange)
        }
    }
    fn new_inner(i2c: I2C, address: u8) -> Self;
    fn new(i2c: I2C, address: u8) -> Result<Self, OutOfRange>
    where
        Self: Sized,
    {
        Self::address_check(address)?;
        Ok(Self::new_inner(i2c, address))
    }
    fn init_inner(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        Ok(self)
    }
    fn init(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        self.init_inner()
    }
}

pub trait WithDelay<I2C: I2c, DELAY: DelayUs, T> {
    fn address_check(address: u8) -> Result<(), OutOfRange> {
        if (0x08..=0x77).contains(&address) {
            Ok(())
        } else {
            Err(OutOfRange)
        }
    }
    fn new_inner(i2c: I2C, address: u8, delay: DELAY) -> Self;
    fn new(i2c: I2C, address: u8, delay: DELAY) -> Result<Self, OutOfRange>
    where
        Self: Sized,
    {
        Self::address_check(address)?;
        Ok(Self::new_inner(i2c, address, delay))
    }
    fn init_inner(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        Ok(self)
    }
    fn init(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        self.init_inner()
    }
}

pub mod p1;
pub mod p10;
// pub mod p11;
pub mod p12;
pub mod p13;
pub mod p14;
// pub mod p15;
pub mod p16;
pub mod p18;
pub mod p19;
// pub mod p2;
pub mod p21;
pub mod p22;
pub mod p23;
pub mod p26;
pub mod p27;
pub mod p29;
pub mod p3;
pub mod p30;
pub mod p7;
