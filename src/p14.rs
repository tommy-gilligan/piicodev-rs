//! # Unofficial Rust Driver for PiicoDev OLED Display
//!
//! ## External Links
//!
//! - [Official Hardware Repository]
//! - [Official MicroPython Repository]
//! - [Official Product Site]
//! - [Datasheet]
//!
//! [Official Hardware Repository]: https://github.com/CoreElectronics/CE-PiicoDev-OLED-SSD1306/tree/9589dfa21c6f25eb7eae1e51cee97ff6fd2c235f
//! [Official MicroPython Repository]: https://github.com/CoreElectronics/CE-PiicoDev-SSD1306-MicroPython-Module/tree/eba37787ef6630fc93784da8dc7a999cfa4f1d0b
//! [Official Product Site]: https://piico.dev/p14
//! [Datasheet]: https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf
use embedded_graphics::{
    draw_target::DrawTarget, geometry::Size, pixelcolor::BinaryColor, prelude::OriginDimensions,
    Pixel,
};
use embedded_hal::i2c::I2c;

const SET_COL_ADDR: u8 = 0x21;
const SET_PAGE_ADDR: u8 = 0x22;
const WIDTH: u8 = 128;
const HEIGHT: u8 = 64;
const PAGES: u8 = HEIGHT / 8;
const BUFFER_SIZE: usize = PAGES as usize * WIDTH as usize;
const INIT_COMMANDS: [u8; 27] = [
    // set disp
    0xAE,
    // set mem addr
    0x20,
    0x00,
    // set disp start line
    0x40,
    // set seg mremap
    0xA0 | 0x01,
    // set mux ratio
    0xA8,
    (HEIGHT - 1),
    // set com out dir
    0xC0 | 0x08,
    // set disp offset
    0xD3,
    0x00,
    // set com pin cfg
    0xDA,
    0x12,
    // set disp clk div
    0xD5,
    0x80,
    // set precharge
    0xD9,
    0xF1,
    // set vcom desel
    0xDB,
    0x30,
    // set contrast
    0x81,
    0xFF,
    // set entire on
    0xA4,
    // set norm inv
    0xA6,
    // set iref select
    0xAD,
    0x30,
    // set charge pump
    0x8D,
    0x14,
    // set disp
    0xAE | 0x01,
];

pub struct P14<I2C> {
    i2c: I2C,
    address: u8,
    framebuffer: [u8; BUFFER_SIZE],
}

use crate::Driver;
impl<I2C: I2c> Driver<I2C> for P14<I2C> {
    fn new_inner(i2c: I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
            framebuffer: [0; BUFFER_SIZE],
        }
    }
}

impl<I2C: I2c> P14<I2C> {
    pub fn init(mut self) -> Result<Self, I2C::Error> {
        for command in INIT_COMMANDS {
            self.i2c.write(self.address, &[0x80, command])?;
        }
        Ok(self)
    }

    /// # Errors
    pub fn show(&mut self) -> Result<(), I2C::Error> {
        let x0: u8 = 0;
        let x1: u8 = WIDTH - 1;
        self.i2c.write(self.address, &[0x80, SET_COL_ADDR])?;
        self.i2c.write(self.address, &[0x80, x0])?;
        self.i2c.write(self.address, &[0x80, x1])?;
        self.i2c.write(self.address, &[0x80, SET_PAGE_ADDR])?;
        self.i2c.write(self.address, &[0x80, 0])?;
        self.i2c.write(self.address, &[0x80, PAGES - 1])?;

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
            if let Ok((x @ 0..=127_u32, y @ 0..=63_u32)) = coord.try_into() {
                let mask: u8 = 1 << (y % 8);
                let index: usize = x as usize + (y as usize / 8) * (WIDTH as usize);

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
mod test {
    extern crate std;
    use std::vec;
    extern crate embedded_hal;
    extern crate embedded_hal_mock;
    use embedded_graphics::{
        geometry::{OriginDimensions, Size},
        pixelcolor::BinaryColor,
        prelude::*,
        primitives::{PrimitiveStyle, Rectangle},
    };
    use embedded_hal_mock::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    use crate::p14::P14;
    use crate::Driver;

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

        P14::new(i2c, 0x3C).unwrap().init().unwrap();
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
