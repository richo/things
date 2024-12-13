#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;

use arduino_hal::prelude::*;

use core::sync::atomic::{AtomicBool, Ordering};

use embedded_hal::i2c::{self, Operation};
use arduino_hal::hal::port::{Pin, PD6};
use arduino_hal::hal::port::mode::{Input, Output, PullUp};
use arduino_hal::i2c::{Direction as I2cDirection};
use embedded_hal::digital::InputPin;

use unflappable::{debouncer_uninit, Debouncer, default::ActiveLow};

mod timer;
use timer::DEBOUNCER;

const MUX_ADDR: u8 = 0x44;
const SHIFT_CUT_DURATION: u16 = 50; // ms

static S_SHIFT: AtomicBool = AtomicBool::new(false);
static S_LAUNCH: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Eq, PartialEq)]
enum Source {
    LiveThrottle,
    Launch,
    QS,
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
    // TODO(richo): Ditch this once we do a rev2 board
    addr: u8,
}

impl<I2C: i2c::I2c> Router<I2C> {
    pub fn new(mut i2c: I2C, addr: u8) -> Self {
        i2c.write(addr, &[Source::LiveThrottle.mask()]);

        Self {
            i2c,
            state: Source::LiveThrottle,
            addr,
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
        self.i2c.transaction(self.addr, &mut ops)
    }

    /// Blocks for the whole duration of the shift cut, so that state probably doesn't actually
    /// need to be managed by the Source enum?
    pub fn shift(&mut self) -> Result<(), I2C::Error> {
        let old = self.state.mask();
        let both = [old & Source::QS.mask()];
        let old_mask = [old];
        let qs_mask = [Source::QS.mask()];

        let mut ops = [
            Operation::Write(&both),
            Operation::Write(&qs_mask),
        ];
        self.i2c.transaction(self.addr, &mut ops)?;

        arduino_hal::delay_ms(50);

        let mut ops = [
            Operation::Write(&both),
            Operation::Write(&old_mask),
        ];
        self.i2c.transaction(self.addr, &mut ops)
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
    let mut debounced_qs = unsafe { DEBOUNCER.init(d6) }.unwrap() ;
	let d7 = pins.d7.into_pull_up_input();

    let _ = ufmt::uwriteln!(serial, "INCREMENT: {}", timer::MILLIS_INCREMENT);



    let mut i2c = arduino_hal::I2c::new(
        dp.TWI,
        sda,
        scl,
        50000,
        );

    let mut led = pins.d13.into_output();
    let mut found = None;
    // This is a gigantic hack because some dumbass left the address pins floating.
    // Instead of pinging just our device and calling it good, we check all four places it could
    // be.

    for i in 0x44 ..= 0x47 {
        if i2c.ping_device(i, I2cDirection::Write).unwrap_or(false) {
            led.set_high();
            found = Some(i);
            let _ = ufmt::uwriteln!(serial, "{:x}: found i2c device", i);
        } else {
            let _ = ufmt::uwriteln!(serial, "{:x}: no i2c device found", i);
        }
    }

    let addr = match found {
        None => loop { arduino_hal::delay_ms(1000); },
        Some(a) => a,
    };

    let mut router = Router::new(i2c, addr);

    timer::timer_init(dp.TC0);
    // SAFETY: Interrupts enabled after this
    unsafe { avr_device::interrupt::enable() };

    let _ = ufmt::uwriteln!(serial, "millis: {}", timer::millis());
    arduino_hal::delay_ms(126);
    let _ = ufmt::uwriteln!(serial, "millis: {}", timer::millis());

    let mut shift = false;

    loop {
        let launch_button = d7.is_low();
        if launch_button { // S_LAUNCH.load(Ordering::Relaxed);
            router.update(Source::Launch)
        } else {
            router.update(Source::LiveThrottle)
        }

        match shift {
            false => {
                if debounced_qs.is_low().unwrap() {
                    router.shift();
                    shift = true;
                }
            },
            true => {
                if debounced_qs.is_high().unwrap() {
                    shift = false;
                }
            }
        }
    }
}
