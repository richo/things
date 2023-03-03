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
    error::Error as DisplayError,
};

pub mod millis;
pub mod brews;
mod formatting;

pub enum Switch {
    Brew,
    NextCancel,
}

use formatting::BoundDisplay;

pub trait Brew {
    fn run(&self, silvia: &mut Silvia) -> Conclusion {
        let res = self.brew(silvia);
        // Confirm all the relays are closed.
        silvia.valve_off();
        silvia.brew_off();
        res
    }

    fn name(&self) -> &'static str;

    /// The main function which interacts with the machine to brew.
    ///
    /// You can safely return without turning off pumps and valves, etc, the frameowrk will take
    /// care of resetting things for you.
    fn brew(&self, silvia: &mut Silvia) -> Conclusion;
}

type Display = HD44780<FourBitBus<Pin<Output, PB4>, Pin<Output, PB3>, Pin<Output, PD6>, Pin<Output, PD5>, Pin<Output, PD4>, Pin<Output, PD3>>>;
#[cfg(feature = "logging")]
type Serial = arduino_hal::usart::Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

#[derive(Clone, Copy)]
pub enum StopReason {
    Brew,
    Cancel,
    Either,
    None,
}

pub struct Silvia {
    #[cfg(feature = "logging")]
    serial: Serial,
    lcd: Display,
    delay: arduino_hal::Delay,
    pump: Pin<Output, PB1>,
    valve: Pin<Output, PB0>,

    brew: Pin<Input<PullUp>, PC4>,
    nextcancel: Pin<Input<PullUp>, PC5>,

    led: Pin<Output, PB5>,

    current: brews::BrewContainer,
}

impl Silvia {
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
        let nextcancel =  pins.a5.into_pull_up_input();

        // relays
        let pump = pins.d9.into_output();
        let valve = pins.d8.into_output();

        let current = brews::BrewContainer::default();

        let mut res = Silvia {
            #[cfg(feature = "logging")]
            serial,
            lcd,
            delay,
            pump,
            valve,
            brew,
            nextcancel,
            led,

            current,
        };
        res.reinit();
        res
    }

    pub fn reinit(&mut self) {
        self.show_current_brew_name();
        self.pump.set_low();
        self.valve.set_low();
    }

    #[cfg(feature = "logging")]
    pub fn serial(&mut self) -> &mut Serial {
        &mut self.serial
    }

    pub fn log(&mut self, _msg: &str) {
        #[cfg(feature = "logging")]
        let _ = ufmt::uwriteln!(self.serial, "{}",  _msg);
    }

    pub fn do_brew(&mut self) -> Conclusion {
        let b = self.current;
        let res = b.get().brew(self);
        // Turn off all the switches
        self.brew_off();
        self.valve_off();
        res
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

    pub fn nextcancel_switch(&mut self) -> bool {
        self.nextcancel.is_low()
    }

    pub fn led(&mut self) -> &mut Pin<Output, PB5> {
        &mut self.led
    }

    pub fn current_brew(&self) -> brews::BrewContainer {
        self.current
    }

    pub fn next_brew(&mut self) {
        self.current = self.current.next();
        self.show_current_brew_name();
    }

    pub fn show_current_brew_name(&mut self) -> Result<(), DisplayError> {
        self.show_brew_name(self.current.get().name())
    }

    pub fn show_brew_name(&mut self, name: &'static str) -> Result<(), DisplayError> {
        let bytes = pad_str(name);
        self.lcd.set_cursor_pos(40, &mut self.delay)?;
        self.lcd.write_bytes(&bytes, &mut self.delay)
    }

    pub fn write_goal(&mut self, time: u32) -> Result<(), DisplayError> {
        self.write_formatted_time(time, true)
    }

    pub fn write_time(&mut self, time: u32) -> Result<(), DisplayError> {
        self.write_formatted_time(time, false)
    }

    pub fn write_formatted_time(&mut self, time: u32, second: bool) -> Result<(), DisplayError> {
        let secs = time / 1000;
        let tenths = (time % 1000) / 100;

        let mut pos = 12;
        if secs < 10 {
            pos += 1
        }
        if second {
            pos += 40
        }

        self.lcd.set_cursor_pos(pos, &mut self.delay)?;
        let mut lcd = BoundDisplay { display: &mut self.lcd, delay: &mut self.delay };
        ufmt::uwriteln!(lcd, "{}.{}", secs, tenths)
    }

    pub fn write_title(&mut self, title: &str) -> Result<(), DisplayError> {
        self.log(title);
        self.lcd.set_cursor_pos(0, &mut self.delay)?;
        let bytes = pad_str(title);
        self.lcd.write_bytes(&bytes, &mut self.delay)
    }

    pub fn report(&mut self, op: Operation) -> Result<(), DisplayError> {
        if let Some(msg) = op.name {
            self.write_title(msg)?;
        }

        self.write_time(op.time)
    }

    pub fn display<'a>(&'a self) -> &'a Display {
        &self.lcd
    }

    pub fn millis(&mut self) -> u32 {
        millis::millis()
    }

    #[inline(always)]
    pub fn delay_ms(&self, time: u16) {
        arduino_hal::delay_ms(time)
    }

    // TODO(richo) This is a hack but for now either button can cancel
    fn unless(&mut self, reason: StopReason) -> bool {
        match reason {
            StopReason::Brew => self.brew.is_low(),
            StopReason::Cancel => self.nextcancel.is_low(),
            StopReason::Either => self.brew.is_low() || self.nextcancel.is_low(),
            StopReason::None => false,
        }
    }

    pub fn until_unless(&mut self, op: &'static str, millis: u16, stop: StopReason) -> Conclusion {
        // TODO(richo) Show goal time in the lower right?
        discard(self.write_title(op));
        // self.write_goal(millis as u32);

        let start = millis::millis();
        let target = start + millis as u32;
        while millis::millis() < target {
            if self.unless(stop) {
                // Wait until the condition clears
                while self.unless(stop) {
                    arduino_hal::delay_ms(RESOLUTION);
                }
                return Conclusion::time(millis::millis() - start);
            }
            discard(self.write_time(millis::millis() - start));
            arduino_hal::delay_ms(RESOLUTION);
        }
        Ok(())
    }

}

#[inline(always)]
/// Discard a Result. This is for the various Display related functions that we don't want to block
/// on. TODO(richo) I think the display being write only means these are functionally infallible
/// anyway.
fn discard(_: Result<(), DisplayError>) {
}


pub fn spin_wait() {
    arduino_hal::delay_ms(100);
}

pub struct Operation {
    name: Option<&'static str>,
    time: u32,
}

pub trait OperationExt: Sized {
    fn interrupted(name: &'static str, time: u32) -> Self;


    fn time(time: u32) -> Self;
}

impl OperationExt for Result<(), Operation> {
    fn interrupted(name: &'static str, time: u32) -> Self {
        Err(Operation { name: Some(name), time})
    }


    fn time(time: u32) -> Self {
        Err(Operation { name: None, time })
    }
}


/// Either Ok, or Err(millis the operation ran for)
pub type Conclusion = Result<(), Operation>;

const RESOLUTION: u16 = 100;
/// Pad a string out to 16 characters, in order to make it consume a full line
fn pad_str(msg: &str) -> [u8; 16] {
    let mut ary = [b' '; 16];
    ary[0..msg.len()].copy_from_slice(msg.as_bytes());
    ary
}

