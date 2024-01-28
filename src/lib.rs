#![no_std]
#![doc = include_str!("../README.md")]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetAddressError<E> {
    I2cError(E),
    ArgumentError,
}
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

/// Some PiicoDev devices include an onboard Atmel MCU that manages access to the device.  These
/// devices share some functionality in common.  [`Atmel`] groups these together.
pub trait Atmel<I2C: I2c> {
    /// Gets whether or not the the LED is powered
    fn get_led(&mut self) -> Result<bool, I2C::Error>;
    /// Sets whether or not the the LED is powered
    fn set_led(&mut self, on: bool) -> Result<(), I2C::Error>;
    /// Gets the version of the Atmel firmware as a (major, minor) tuple.
    fn firmware(&mut self) -> Result<(u8, u8), I2C::Error>;
    /// Sets the I2C address of the device.  A new driver instance must be created to access the
    /// device when its address is changed.  Some devices also have onboard DIP switches for
    /// setting a device address.  The interaction between DIP switches and setting the address
    /// through the driver should be determined experimentally or through interrogation of firmware
    /// source.
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

    /// The entry point for a [`Driver`].  Expects [`I2c`] (obtainable from target platform HAL)
    /// and an I2C device address in the range `0x08..=0x77`.  This provides a handle that does not
    /// initialize the hardware.  Initialization is deferred to [`Driver::init`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`]: address is ouside of the allowed range `0x08..=0x77`
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

    /// Initializes the hardware.  This initialization is usually required prior to interacting
    /// with the device.  Some devices do not need initializing before use.  Calling
    /// [`Driver::init`] will be enforced through state types in the future.
    ///
    /// # Errors
    ///
    /// [`T`]: a device dependent error type for any problems encountered during initialization.
    fn init(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        self.init_inner()
    }
}

pub trait DriverUsingDelay<I2C: I2c, DELAY: DelayNs, T> {
    fn address_check(address: u8) -> Result<(), OutOfRange> {
        if (0x08..=0x77).contains(&address) {
            Ok(())
        } else {
            Err(OutOfRange)
        }
    }

    fn new_inner(i2c: I2C, address: u8, delay: DELAY) -> Self;

    /// The entry point for a [`DriverUsingDelay`].  Expects [`I2c`] (obtainable from target
    /// platform HAL), an I2C device address in the range `0x08..=0x77` and a [`DelayUs`] (also
    /// usually obtainable from the target platform HAL).  This provides a handle that does not
    /// initialize the hardware.  Initialization is deferred to [`DriverUsingDelay::init`].
    ///
    /// # Errors
    ///
    /// [`OutOfRange`]: address is ouside of the allowed range `0x08..=0x77`
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

    /// Initializes the hardware.  This initialization is usually required prior to interacting
    /// with the device.  Some devices do not need initializing before use.  Calling
    /// [`Driver::init`] will be enforced through state types in the future.
    ///
    /// # Errors
    ///
    /// [`T`]: a device dependent error type for any problems encountered during initialization.
    fn init(self) -> Result<Self, T>
    where
        Self: Sized,
    {
        self.init_inner()
    }
}

pub mod p1;
pub mod p3;
pub mod p30;
pub mod p7;
// pub mod p10;
// pub mod p11;
// pub mod p12;
// pub mod p13;
// pub mod p14;
// pub mod p15;
// pub mod p16;
// pub mod p18;
// pub mod p19;
// pub mod p2;
pub mod p21;
// pub mod p22;
// pub mod p23;
// pub mod p26;
// pub mod p27;
// pub mod p29;
