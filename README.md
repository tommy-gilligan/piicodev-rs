# Unofficial Rust Drivers for PiicoDev

## Supported Devices

Sometimes an extra 'helper' MCU is necessary to add or modify an I^2C interface
to the central component of the design.  These are marked in the table below.

| Purchase Link                  | Driver Link                          | Helper MCU | IC                                                                                                            |
| ------------------------------ | ------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------------- |
| [piico.dev/p1](https://piico.dev/p1)   | [Precision Temperature Sensor](./p1) | ❌         | [TMP117](https://www.ti.com/product/TMP117)                                                                   |
| [piico.dev/p3](https://piico.dev/p3)   | [Ambient Light Sensor](./p3)         | ❌         | [VEML6030](https://www.vishay.com/en/product/84366/)                                                          |
| [piico.dev/p7](https://piico.dev/p7)   | [Laser Distance Sensor](./p7)        | ❌         | [VL53L1X](https://www.st.com/en/imaging-and-photonics-solutions/vl53l1x.html)                                 |
| [piico.dev/p10](https://piico.dev/p10) | [Colour Sensor](./p10)               | ❌         | [VEML6040](https://www.vishay.com/docs/84276/veml6040.pdf)                                                    |
| [piico.dev/p11](https://piico.dev/p11) | [Pressure Sensor](./p11)             | ❌         | [MS5637](https://www.infineon.com/cms/en/product/sensor/pressure-sensors/pressor-iot/?gclid=CjwKCAjw6IiiBhAOEiwALNqncTXLg-VcFVTQ0JKX3Pw2DNB0FBmP0Hbo4GsIiMfW9RfIf8YXXQG48hoCTccQAvD_BwE&gclsrc=aw.ds) |
| [piico.dev/p12](https://piico.dev/p12) | [Capacitive Touch Sensor](./p12)     | ❌         | [CAP1203](https://www.microchip.com/en-us/product/CAP1203)                                                    |
| [piico.dev/p13](https://piico.dev/p13) | [3x RGB LED](./p13)                  | ✅         | [WS2812B](https://cdn-shop.adafruit.com/datasheets/WS2812B.pdf)                                               |
| [piico.dev/p14](https://piico.dev/p14) | [OLED Display](./p14)                | ❌         | [SSD1306](https://cdn-shop.adafruit.com/datasheets/SSD1306.pdf)                                               |
| [piico.dev/p18](https://piico.dev/p18) | [Buzzer](./p18)                      | ✅         | [MLT-8540H](https://datasheet.lcsc.com/lcsc/1811141116_Jiangsu-Huaneng-Elec-MLT-8540H_C95298.pdf)                                  |
| [piico.dev/p21](https://piico.dev/p21) | [Button](./p21)                      | ✅         |                                                                                                               |
| [piico.dev/p22](https://piico.dev/p22) | [Potentiometer](./p22)               | ✅         |                                                                                                               |
| [piico.dev/p27](https://piico.dev/p27) | [Transceiver](./p27)                 | ✅         | [RFM69HCW](https://www.hoperf.com/modules/rf_transceiver/RFM69HCW.html)                                       |
| [piico.dev/p29](https://piico.dev/p29) | [Servo Driver](./p29)                | ❌         | [PCA9685](https://www.nxp.com/products/power-management/lighting-driver-and-controller-ics/led-controllers/16-channel-12-bit-pwm-fm-plus-ic-bus-led-controller:PCA9685) |
| [piico.dev/p30](https://piico.dev/p30) | [Ultrasonic Rangefinder](./p30)      | ✅         | [RCWL1601](https://cdn.sparkfun.com/datasheets/Sensors/Proximity/HCSR04.pdf)                                  |

| Purchase Link                  | Driver Link                          | Helper MCU | IC                                                                                                            |
| ------------------------------ | ------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------------------- |
| [piico.dev/p16](https://piico.dev/p16) | [RFID](./p16)                        | ❌         | [MFRC522](https://www.nxp.com/docs/en/data-sheet/MFRC522.pdf)                                                 |
| [piico.dev/p15](https://piico.dev/p15) | [Magnetometer](./p15)                | ❌         | [QMC6310](https://github.com/CoreElectronics/CE-PiicoDev-Magnetometer-QMC6310/raw/main/Documents/QMC6310.pdf) |
| [piico.dev/p19](https://piico.dev/p19) | [Real-Time Clock](./p19)             | ❌         | [RV3028](https://www.microcrystal.com/fileadmin/Media/Products/RTC/App.Manual/RV-3028-C7_App-Manual.pdf)      |
| [piico.dev/p26](https://piico.dev/p26) | [3-Axis Accelerometer](./p26)        | ❌         | [LIS3DH](https://www.st.com/en/mems-and-sensors/lis3dh.html)                                                  |

Ultrasonic Rangefinder schematic is incomplete
Do all Piico.dev pages link to Github pages: P22, P29
New units for magnetometr
Inconsistent treatment of discontinued products

## Design Decisions

## Getting Started
### Pico
