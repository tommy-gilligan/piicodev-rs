# Unofficial Rust Drivers for PiicoDev

Excellent MicroPython support from official packages.
This here is: these are unofficial Rust drivers based around embedded-hal.
These drivers can probably be used wherever embedded-hal can be used.

## Supported Devices

- [Precision Temperature Sensor](./p1/)
- [Ambient Light Sensor](./p3/)
- [Laser Distance Sensor](./p7/)
- [Color Sensor](./p10/)
- [Pressure Sensor](./p11/)
- [Capacitive Touch Sensor](./p12/)
- [3x RGB LED](./p13/)
- [OLED Display](./p14/)
- [Buzzer](./p18/)
- [Button](./p21/)
- [Potentiometer](./p22/)
- [Transceiver](./p27/)
- [Servo Driver](./p29/)
- [Ultrasonic Rangefinder](./p30/)

## Issues

Sometimes an extra 'helper' MCU is necessary to add or modify an I^2C interface
to the central component of the design.  These are marked in the table below.


Ultrasonic Rangefinder schematic is incomplete
Do all Piico.dev pages link to Github pages: P22, P29
New units for magnetometr
Inconsistent treatment of discontinued products
clippy should try again when config is changed, but only changes when source is changed

## Design Decisions

## Getting Started
### Pico
