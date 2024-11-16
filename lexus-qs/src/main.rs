#![no_std]
#![no_main]

use panic_halt as _;

use embedded_hal::i2c::{self, Operation};
use arduino_hal::prelude::*;
use arduino_hal::i2c::{Direction as I2cDirection};
use arduino_hal::hal::port::{Pin, PB2, PB3, PB1, PB5, PC0, PC1, PC2};

const MUX_ADDR: u8 = 0x44;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Source {
    LiveThrottle,
    Launch,
    QS,
}

#[derive(Clone, Copy)]
enum Channel {
    A,
    B,
}

impl Source {
    const fn mask(&self) -> u8 {
        match self {
            Source::LiveThrottle => 0b00010001 << 0,
            Source::Launch => 0b00010001 << 1,
            Source::QS => 0b00010001 << 2,
        }
    }
}

struct Router<I2C> {
    i2c: I2C,
    state: Source,
}

impl<I2C: i2c::I2c> Router<I2C> {
    pub fn new(mut i2c: I2C) -> Self {
        i2c.write(MUX_ADDR, &[Source::LiveThrottle.mask()]);

        Self {
            i2c,
            state: Source::LiveThrottle,
        }
    }

    pub fn update(&mut self, state: Source) {
        if state != self.state {
            self.set_source(state);
        }
    }

    pub fn set_source(&mut self, new: Source) -> Result<(), I2C::Error> {
        let both = [self.state.mask() & new.mask()];
        let new_mask = [new.mask()];
        let mut ops = [
            Operation::Write(&both),
            Operation::Write(&new_mask),
        ];

        self.state = new;
        self.i2c.transaction(MUX_ADDR, &mut ops)
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

	let sda = pins.a4.into_pull_up_input();
	let scl = pins.a5.into_pull_up_input();

	let mut _d5 = pins.d5.into_output();
    _d5.set_low();
	let d6 = pins.d6.into_pull_up_input();

    let _ = ufmt::uwriteln!(serial, "hi");



    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        sda,
        scl,
        50000,
        );

    let mut led = pins.d13.into_output();
    let mut found = false;
    if i2c.ping_device(MUX_ADDR, I2cDirection::Write).unwrap_or(false) {
        led.set_high();
        found = true;
    }

    let mut router = Router::new(i2c);

    if found {
        loop {
            let launch_button = d6.is_low();
            if launch_button {
                router.update(Source::Launch)
            } else {
                router.update(Source::LiveThrottle)
            }

            arduino_hal::delay_ms(50);
        }
    } else {
        loop {
            arduino_hal::delay_ms(1000);
        }
    }
}
