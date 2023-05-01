#![no_std]
use embedded_graphics::prelude::OriginDimensions;
use embedded_graphics::Pixel;
use embedded_graphics::{draw_target::DrawTarget, geometry::Size, pixelcolor::BinaryColor};
use embedded_hal::i2c::I2c;

const SET_CONTRAST: u8 = 0x81;
const SET_ENTIRE_ON: u8 = 0xA4;
const SET_NORM_INV: u8 = 0xA6;
const SET_DISP: u8 = 0xAE;
const SET_MEM_ADDR: u8 = 0x20;
const SET_COL_ADDR: u8 = 0x21;
const SET_PAGE_ADDR: u8 = 0x22;
const SET_DISP_START_LINE: u8 = 0x40;
const SET_SEG_REMAP: u8 = 0xA0;
const SET_MUX_RATIO: u8 = 0xA8;
const SET_IREF_SELECT: u8 = 0xAD;
const SET_COM_OUT_DIR: u8 = 0xC0;
const SET_DISP_OFFSET: u8 = 0xD3;
const SET_COM_PIN_CFG: u8 = 0xDA;
const SET_DISP_CLK_DIV: u8 = 0xD5;
const SET_PRECHARGE: u8 = 0xD9;
const SET_VCOM_DESEL: u8 = 0xDB;
const SET_CHARGE_PUMP: u8 = 0x8D;

const WIDTH: usize = 128;
const HEIGHT: usize = 64;
const PAGES: usize = HEIGHT / 8;
const BUFFER_SIZE: usize = PAGES * WIDTH;

pub struct P14<I2C> {
    i2c: I2C,
    address: u8,
    framebuffer: [u8; BUFFER_SIZE],
}

impl<I2C: I2c> P14<I2C> {
    /// # Errors
    pub fn new(i2c: I2C, address: u8) -> Result<Self, I2C::Error> {
        let mut res = Self {
            i2c,
            address,
            framebuffer: [0; BUFFER_SIZE],
        };

        for cmd in [
            SET_DISP,
            SET_MEM_ADDR,
            0x00,
            SET_DISP_START_LINE,
            SET_SEG_REMAP | 0x01,
            SET_MUX_RATIO,
            (HEIGHT - 1) as u8,
            SET_COM_OUT_DIR | 0x08,
            SET_DISP_OFFSET,
            0x00,
            SET_COM_PIN_CFG,
            0x12,
            SET_DISP_CLK_DIV,
            0x80,
            SET_PRECHARGE,
            0xF1,
            SET_VCOM_DESEL,
            0x30,
            SET_CONTRAST,
            0xFF,
            SET_ENTIRE_ON,
            SET_NORM_INV,
            SET_IREF_SELECT,
            0x30,
            SET_CHARGE_PUMP,
            0x14,
            SET_DISP | 0x01,
        ] {
            res.i2c.write(res.address, &[0x80, cmd])?;
        }
        Ok(res)
    }

    /// # Errors
    pub fn show(&mut self) -> Result<(), I2C::Error> {
        let x0: usize = 0;
        let x1: usize = WIDTH - 1;
        self.i2c.write(self.address, &[0x80, SET_COL_ADDR])?;
        self.i2c.write(self.address, &[0x80, x0 as u8])?;
        self.i2c.write(self.address, &[0x80, x1 as u8])?;
        self.i2c.write(self.address, &[0x80, SET_PAGE_ADDR])?;
        self.i2c.write(self.address, &[0x80, 0])?;
        self.i2c.write(self.address, &[0x80, PAGES as u8 - 1])?;

        let mut i2c_buffer: [u8; 1025] = [0x40; 1025];
        for (i, val) in self.framebuffer.iter().enumerate() {
            i2c_buffer[i + 1] = *val;
        }

        self.i2c.write(self.address, &i2c_buffer)?;
        Ok(())
    }
}

impl<I2C: I2c> DrawTarget for P14<I2C> {
    type Color = BinaryColor;
    type Error = core::convert::Infallible;

    /// # Errors
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if let Ok((x @ 0..=127i32, y @ 0..=63i32)) = coord.try_into() {
                let mask: u8 = 1 << (y % 8);
                let index: usize = x as usize + (y as usize / 8) * WIDTH;

                if color.is_on() {
                    self.framebuffer[index] |= mask;
                } else {
                    self.framebuffer[index] &= !mask;
                }
            }
        }

        Ok(())
    }
}

impl<I2C: I2c> OriginDimensions for P14<I2C> {
    fn size(&self) -> Size {
        Size::new(WIDTH.try_into().unwrap(), HEIGHT.try_into().unwrap())
    }
}

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
#[macro_use]
extern crate std;

#[cfg(all(test, not(all(target_arch = "arm", target_os = "none"))))]
mod test {
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_graphics::{
        geometry::{OriginDimensions, Size},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::{PrimitiveStyle, Rectangle},
    };
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::P14;

    #[test]
    pub fn new() {
        let expectations = [
            I2cTransaction::write(0x3C, vec![0x80, 174]),
            I2cTransaction::write(0x3C, vec![0x80, 32]),
            I2cTransaction::write(0x3C, vec![0x80, 0]),
            I2cTransaction::write(0x3C, vec![0x80, 64]),
            I2cTransaction::write(0x3C, vec![0x80, 161]),
            I2cTransaction::write(0x3C, vec![0x80, 168]),
            I2cTransaction::write(0x3C, vec![0x80, 63]),
            I2cTransaction::write(0x3C, vec![0x80, 200]),
            I2cTransaction::write(0x3C, vec![0x80, 211]),
            I2cTransaction::write(0x3C, vec![0x80, 0]),
            I2cTransaction::write(0x3C, vec![0x80, 218]),
            I2cTransaction::write(0x3C, vec![0x80, 18]),
            I2cTransaction::write(0x3C, vec![0x80, 213]),
            I2cTransaction::write(0x3C, vec![0x80, 128]),
            I2cTransaction::write(0x3C, vec![0x80, 217]),
            I2cTransaction::write(0x3C, vec![0x80, 241]),
            I2cTransaction::write(0x3C, vec![0x80, 219]),
            I2cTransaction::write(0x3C, vec![0x80, 48]),
            I2cTransaction::write(0x3C, vec![0x80, 129]),
            I2cTransaction::write(0x3C, vec![0x80, 255]),
            I2cTransaction::write(0x3C, vec![0x80, 164]),
            I2cTransaction::write(0x3C, vec![0x80, 166]),
            I2cTransaction::write(0x3C, vec![0x80, 173]),
            I2cTransaction::write(0x3C, vec![0x80, 48]),
            I2cTransaction::write(0x3C, vec![0x80, 141]),
            I2cTransaction::write(0x3C, vec![0x80, 20]),
            I2cTransaction::write(0x3C, vec![0x80, 175]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        P14::new(i2c, 0x3C).unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn show() {
        let mut v = vec![0x0f; 1025];
        v[0] = 0x40;
        let expectations = [
            I2cTransaction::write(0x3C, vec![0x80, 0x21]),
            I2cTransaction::write(0x3C, vec![0x80, 0]),
            I2cTransaction::write(0x3C, vec![0x80, 127]),
            I2cTransaction::write(0x3C, vec![0x80, 0x22]),
            I2cTransaction::write(0x3C, vec![0x80, 0]),
            I2cTransaction::write(0x3C, vec![0x80, 7]),
            I2cTransaction::write(0x3C, v),
        ];

        let i2c = I2cMock::new(&expectations);
        let mut i2c_clone = i2c.clone();

        let mut p14 = P14 {
            i2c,
            address: 0x3C,
            framebuffer: [0x0f; 1024],
        };
        p14.show().unwrap();
        i2c_clone.done();
    }

    #[test]
    pub fn size() {
        let i2c = I2cMock::new(&[]);
        let p14 = P14 {
            i2c,
            address: 0x3C,
            framebuffer: [0; 1024],
        };
        assert_eq!(p14.size(), Size::new(128, 64));
    }

    #[test]
    pub fn draw_target() {
        let i2c = I2cMock::new(&[]);
        let mut p14 = P14 {
            i2c,
            address: 0x3C,
            framebuffer: [0; 1024],
        };

        let fill = PrimitiveStyle::with_fill(BinaryColor::On);
        Rectangle::new(Point::new(0, 0), Size::new(16, 2))
            .into_styled(fill)
            .draw(&mut p14)
            .unwrap();

        assert_eq!(
            p14.framebuffer,
            [
                3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
    }
}
