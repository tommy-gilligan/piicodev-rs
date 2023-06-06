#![feature(int_roundings)]
#![feature(lint_reasons)]
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetAddressError<E> {
    I2cError(E),
    ArgumentError,
}
use embedded_hal::i2c::I2c;
trait Atmel<I2C: I2c> {
    fn get_led(&mut self) -> Result<bool, I2C::Error>;
    fn set_led(&mut self, on: bool) -> Result<(), I2C::Error>;
    fn firmware(&mut self) -> Result<(u8, u8), I2C::Error>;
    fn whoami(&mut self) -> Result<u16, I2C::Error>;
    fn set_address(&mut self, new_address: u8) -> Result<(), SetAddressError<I2C::Error>>;
}

pub mod p1;
pub mod p10;
pub mod p11;
pub mod p12;
pub mod p13;
pub mod p14;
pub mod p15;
pub mod p16;
pub mod p18;
pub mod p19;
pub mod p2;
pub mod p21;
pub mod p22;
pub mod p23;
pub mod p26;
pub mod p27;
pub mod p29;
pub mod p3;
pub mod p30;
pub mod p7;
