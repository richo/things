#![no_std]
#![no_main]

use panic_halt as _;

use embedded_hal::i2c;
use arduino_hal::prelude::*;
use arduino_hal::i2c::{I2c, Direction};
use arduino_hal::hal::port::{Pin, PB2, PB3, PB1, PB5, PC0, PC1, PC2};

const MUX_ADDR: u8 = 0x44;

struct MuxDriver<I2C> {
    c: I2C,
}

enum Source {
    LiveThrottle,
    Launch,
    QS,
}

impl<I2C: i2c::I2c> MuxDriver<I2C> {
    pub fn new(c: I2C) -> Self {
        Self { c }
    }

    pub fn set_source(&mut self, s: Source) {
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */
	let sda = pins.a4.into_pull_up_input();
	let scl = pins.a5.into_pull_up_input();

    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        sda,
        scl,
        50000,
        );

    let mut found = false;
    if let Ok(r) = i2c.ping_device(0x44, Direction::Write) {
        found = r
    }




    fn set_channel<I: i2c::I2c> (i2c: &mut I, mask: u8) {
        // Set to both, so it never sees an open circuit
        i2c.write(0x44, &[0b00000011]);
        i2c.write(0x44, &[mask]);
    }

    let mut led = pins.d13.into_output();

    if found {
        let mut channel: u8 = 0;

        loop {
            // Set a channel
            let mask = 1 << channel;
            set_channel(&mut i2c, mask);
            led.toggle();
            arduino_hal::delay_ms(1000);
            channel += 1;
            if channel == 2 {
                channel = 0;
            }
        }
    } else {
        loop {
            arduino_hal::delay_ms(1000);
        }
    }
}
