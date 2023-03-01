#![no_std]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
pub use arduino_hal::prelude::*;
use arduino_hal::hal::pac::USART0;
// TODO(richo) pare these down once we're sure we have everything we need.
#[allow(unused_imports)]
use arduino_hal::hal::port::{Pin, PB2, PB3, PB4, PD6, PD5, PD4, PD3, PC4, PC5, PD1, PD0, PB0, PB1, PB5};
use arduino_hal::hal::port::mode::{Input, Output, PullUp};

use hd44780_driver::{
    HD44780,
    bus::FourBitBus,
};

pub mod millis;


// type PUMP = Pin<Output, PB2>;
// type VALVE = Pin<Output, PB3>;


type Display = HD44780<FourBitBus<Pin<Output, PB4>, Pin<Output, PB3>, Pin<Output, PD6>, Pin<Output, PD5>, Pin<Output, PD4>, Pin<Output, PD3>>>;
type Serial = arduino_hal::usart::Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

pub struct Devices {
    serial: Serial,
    lcd: Display,
    delay: arduino_hal::Delay,
    pump: Pin<Output, PB1>,
    valve: Pin<Output, PB0>,

    brew: Pin<Input<PullUp>, PC4>,
    backflush: Pin<Input<PullUp>, PC5>,

    led: Pin<Output, PB5>,
}

impl Devices {
    pub fn new() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();

        millis::millis_init(dp.TC0);
        unsafe { avr_device::interrupt::enable() };

        let pins = arduino_hal::pins!(dp);
        let serial = arduino_hal::default_serial!(dp, pins, 57600);

        // Display
        let rs = pins.d12.into_output();
        let e = pins.d11.into_output();

        let d4 = pins.d6.into_output();
        let d5 = pins.d5.into_output();
        let d6 = pins.d4.into_output();
        let d7 = pins.d3.into_output();
        let mut delay = arduino_hal::Delay::new();
        let lcd = hd44780_driver::HD44780::new_4bit(
            rs, e,
            d4, d5, d6, d7,
            &mut delay,
        ).unwrap();

        // Led
        let led = pins.d13.into_output();

        // Switches

        let brew =  pins.a4.into_pull_up_input();
        let backflush =  pins.a5.into_pull_up_input();

        // relays
        let pump = pins.d9.into_output();
        let valve = pins.d8.into_output();

        let mut res = Devices {
            serial,
            lcd,
            delay,
            pump,
            valve,
            brew,
            backflush,
            led,
        };
        res.reinit();
        res
    }

    pub fn reinit(&mut self) {
        self.pump.set_low();
        self.valve.set_low();
    }

    pub fn serial(&mut self) -> &mut Serial {
        &mut self.serial
    }

    pub fn log(&mut self, msg: &str) {
        let _ = ufmt::uwriteln!(self.serial, "{}",  msg);
    }

    pub fn brew_on(&mut self) {
        self.pump.set_high()
    }

    pub fn brew_off(&mut self) {
        self.pump.set_low()
    }

    pub fn valve_on(&mut self) {
        self.valve.set_high()
    }

    pub fn valve_off(&mut self) {
        self.valve.set_low()
    }

    pub fn brew_switch(&mut self) -> bool {
        self.brew.is_low()
    }

    pub fn backflush_switch(&mut self) -> bool {
        self.backflush.is_low()
    }

    pub fn led(&mut self) -> &mut Pin<Output, PB5> {
        &mut self.led
    }

    pub fn display<'a>(&'a self) -> &'a Display {
        &self.lcd
    }

    pub fn display_str(&mut self, msg: &str) -> () {
        // TODO(richo) is it actually possible to handle errors here?
        let _ = self.lcd.clear(&mut self.delay);
        let _ = self.lcd.write_str(msg, &mut self.delay);
    }

    pub fn millis(&mut self) -> u32 {
        millis::millis()
    }

    pub fn run_infuse_and_brew(&mut self) -> Conclusion {
        // Enable valve
        self.valve.set_high();
        // Pulse pump on and off for 300/200 3 times
        for t in [200, 300, 400] {
            self.pump.set_high();
            let infuse = |time| { let _ = ufmt::uwriteln!(self.serial, "infuse {}",  time); };
            if let Conclusion::Interrupted(i) = until_unless(t, || self.brew.is_low(), infuse) {
                self.pump.set_low();
                return Conclusion::Interrupted(i);
            }
            self.pump.set_low();
            let wait = |time| { let _ = ufmt::uwriteln!(self.serial, "inwait {}",  time); };
            if let Conclusion::Interrupted(i) = until_unless(200, || self.brew.is_low(), wait) {
                self.pump.set_low();
                return Conclusion::Interrupted(i);
            }
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        self.valve.set_high();
        self.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        let brew = |time| { let _ = ufmt::uwriteln!(self.serial, "brew {}",  time); };
        until_unless(35000, || self.brew.is_low(), brew)
    }

    /// Confirm the solenoid is closed, then run the brew pump for some configurable number of millies,
    /// then turn off the pump and the solenoid
    pub fn run_brew(&mut self) -> Conclusion {
        // Infuse leaves the valve closed, but we'll double check
        self.valve.set_high();
        self.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        // TODO(richo) Again holy shit this should be an interrupt thing
        until_unless(3500, || self.brew.is_low(), |_| {})
    }

    #[inline(always)]
    pub fn delay_ms(&self, time: u16) {
        arduino_hal::delay_ms(time)
    }

    pub fn run_infuse(&mut self) -> Conclusion {
        // Infuse the puck by closing the solenoid and running the pump, but do not open the valve when
        // finished.
        self.valve.set_high();
        self.pump.set_high();
        if let Conclusion::Interrupted(i) = until_unless(INFUSE_MILLIS, || self.brew.is_low(), |_| {}) {
            self.pump.set_low();
            self.valve.set_low();
            return Conclusion::Interrupted(i);
        }
        self.pump.set_low();
        until_unless(INFUSE_WAIT_MILLIS, || self.brew.is_low(), |_| {})
    }

    pub fn run_backflush(&mut self) -> Conclusion {


        for _ in 0..BACKFLUSH_REPEATS {
            self.valve.set_high();
            self.pump.set_high();
            let flush = |time| { let _ = ufmt::uwriteln!(self.serial, "flush {}",  time); };
            let res = until_unless(BACKFLUSH_ON_MILLIS, || self.backflush.is_low(), flush);
            self.pump.set_low();
            self.valve.set_low();
            let wait = |time| { let _ = ufmt::uwriteln!(self.serial, "wait {}",  time); };
            if let Conclusion::Finished = res {
                until_unless(BACKFLUSH_PAUSE_MILLIS, || self.backflush.is_low(), wait);
            } else {
                return res
            }
        }
        Conclusion::Finished
    }
}


pub fn spin_wait() {
    arduino_hal::delay_ms(100);
}

/// Turn on the pump and solenoid, wait some configurable number of millis, turn off the pump, wait
/// some configurable number of millis, without opening the 3 way valve.
const INFUSE_MILLIS: u16 = 2000;
const INFUSE_WAIT_MILLIS: u16 = 2500;

// TODO(richo) impl the traits that let me ? this
pub enum Conclusion {
    Finished,
    /// Contains the number of millis into the operation it was interrupted
    Interrupted(u32),
}

const RESOLUTION: u16 = 100;
fn until_unless<F, P>(millis: u16, unless: F, mut progress: P) -> Conclusion
where F: Fn() -> bool,
      P: FnMut(u32) {
    let start = millis::millis();
    let mut target = start + millis as u32;
    while millis::millis() < target {
        if unless() {
            // Wait until the condition clears
            while unless() {
                arduino_hal::delay_ms(RESOLUTION);
            }
            return Conclusion::Interrupted(millis::millis() - start);
        }
        progress(millis::millis() - start);
        arduino_hal::delay_ms(RESOLUTION);
    }
    Conclusion::Finished
}

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 7000;

