#![no_std]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
pub use arduino_hal::prelude::*;
// TODO(richo) pare these down once we're sure we have everything we need.
#[allow(unused_imports)]
use arduino_hal::hal::port::{Pin, PB2, PB3, PB4, PD6, PD5, PD4, PD3, PC4, PC5, PD1, PD0, PB0, PB1, PB5};
use arduino_hal::hal::port::mode::{Input, Output, PullUp};

#[cfg(feature = "logging")]
use arduino_hal::hal::pac::USART0;

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

pub enum Row {
    First,
    Second,
}

impl Into<u8> for Row {
    fn into(self) -> u8 {
        match self {
            Row::First => 0,
            Row::Second => 40
        }
    }
}

use formatting::BoundDisplay;

pub trait Brew {
    const NAME: &'static str;

    fn run(silvia: &mut Silvia) -> Conclusion {
        Self::log(silvia, "starting brew");
        let res = Self::brew(silvia);
        // Confirm all the relays are closed.
        silvia.valve_off();
        silvia.pump_off();
        res
    }

    fn log(_silvia: &mut Silvia, _msg: &'static str) {
        #[cfg(feature = "logging")]
        let _ = ufmt::uwriteln!(_silvia.serial, "{} {}",  Self::NAME, _msg);
    }

    /// The main function which interacts with the machine to brew.
    ///
    /// You can safely return without turning off pumps and valves, etc, the frameowrk will take
    /// care of resetting things for you.
    fn brew(silvia: &mut Silvia) -> Conclusion;
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

pub enum Count {
    Up,
    DownFrom(u32),
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
    pub last: Option<u32>,
}

impl Silvia {
    pub fn new() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();

        millis::millis_init(dp.TC0);
        unsafe { avr_device::interrupt::enable() };

        let pins = arduino_hal::pins!(dp);
        #[cfg(feature = "logging")]
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
            last: None,
        };
        res.reinit();
        res
    }

    pub fn reinit(&mut self) -> Result<(), DisplayError> {
        self.show_current_brew_name()?;
        self.pump.set_low();
        self.valve.set_low();
        Ok(())
    }

    pub fn reset_display(&mut self) -> Result<(), DisplayError> {
        self.lcd.init_4bit(&mut self.delay)?;
        self.reinit()
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
        let b = self.current_brew();
        let res = b.brew(self);
        // Turn off all the switches
        self.pump_off();
        self.valve_off();
        res
    }

    pub fn pump_on(&mut self) {
        self.log("pump on");
        #[cfg(not(feature = "disable-relays"))]
        self.pump.set_high()
    }

    pub fn pump_off(&mut self) {
        self.log("pump off");
        #[cfg(not(feature = "disable-relays"))]
        self.pump.set_low()
    }

    pub fn valve_on(&mut self) {
        self.log("valve on");
        #[cfg(not(feature = "disable-relays"))]
        self.valve.set_high()
    }

    pub fn valve_off(&mut self) {
        self.log("valve off");
        #[cfg(not(feature = "disable-relays"))]
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

    pub fn next_brew(&mut self) -> Result<(), DisplayError> {
        self.current = self.current.next();
        self.show_current_brew_name()
    }

    /// 4 characters worth of extra "scratch space"
    /// This will be overwritten by the elapsed time when the brew ends
    pub fn write_extra(&mut self, extra: &[u8; 4]) -> Result<(), DisplayError> {
        let mut bytes = pad_str(self.current.name(), None);
        bytes[12..].copy_from_slice(extra);
        self.write_buf(&bytes, Row::Second)
    }

    pub fn show_current_brew_name(&mut self) -> Result<(), DisplayError> {
        self.show_brew_name(self.current.name())
    }

    fn show_brew_name(&mut self, name: &'static str) -> Result<(), DisplayError> {
        let bytes = pad_str(name, self.last);
        self.write_buf(&bytes, Row::Second)
    }

    fn write_buf(&mut self, buf: &[u8; 16], row: Row) -> Result<(), DisplayError> {
        self.lcd.set_cursor_pos(row.into(), &mut self.delay)?;
        self.lcd.write_bytes(buf, &mut self.delay)
    }

    pub fn write_time(&mut self, time: u32) -> Result<(), DisplayError> {
        self.write_formatted_time(time)
    }

    pub fn write_formatted_time(&mut self, time: u32) -> Result<(), DisplayError> {
        let secs = time / 1000;
        let tenths = (time % 1000) / 100;

        let mut pos = 12;
        if secs < 10 {
            pos += 1
        }
        self.lcd.set_cursor_pos(pos, &mut self.delay)?;
        let mut lcd = BoundDisplay { display: &mut self.lcd, delay: &mut self.delay };
        ufmt::uwrite!(lcd, "{}.{}", secs, tenths)
    }

    pub fn write_title(&mut self, title: &str) -> Result<(), DisplayError> {
        self.lcd.set_cursor_pos(0, &mut self.delay)?;
        let bytes = pad_str(title, None);
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

    pub fn until_unless(&mut self, op: &'static str, millis: u16, stop: StopReason, count: Count) -> Conclusion {
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
            match count {
                Count::Up => {
                    discard(self.write_time(millis::millis() - start));
                },
                Count::DownFrom(t) => {
                    discard(self.write_time(t - (millis::millis() - start)));
                },
                Count::None => {},
            }
            arduino_hal::delay_ms(RESOLUTION);
        }
        Conclusion::finished(millis as u32)
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
    pub name: Option<&'static str>,
    pub time: u32,
}

pub trait OperationExt: Sized {
    fn interrupted(name: &'static str, time: u32) -> Self;


    fn time(time: u32) -> Self;

    fn finished(time: u32) -> Self;
}

impl OperationExt for Result<Operation, Operation> {
    fn interrupted(name: &'static str, time: u32) -> Self {
        Err(Operation { name: Some(name), time})
    }


    fn time(time: u32) -> Self {
        Err(Operation { name: None, time })
    }

    fn finished(time: u32) -> Self {
        Ok(Operation { name: None, time })
    }
}


/// Either Ok, or Err(millis the operation ran for)
pub type Conclusion = Result<Operation, Operation>;

// TODO(richo) pull this out and ditch ufmt entirely
const RESOLUTION: u16 = 100;
/// Pad a string out to 16 characters, in order to make it consume a full line
fn pad_str(msg: &str, last: Option<u32>) -> [u8; 16] {
    let mut ary = [b' '; 16];
    ary[0..msg.len()].copy_from_slice(msg.as_bytes());
    match last {
        Some(t) if t > 0 => {
            let zero = b'0';
            let secs = t / 1000;
            let tenths = (t % 1000) / 100;
            let mut idx = 16 - 4;

            if secs > 10 {
                ary[idx] = zero + (secs / 10) as u8;
            }

            idx += 1;
            ary[idx] = zero + (secs % 10) as u8;

            idx += 1;
            ary[idx] = b'.';

            idx += 1;
            ary[idx] = zero + tenths as u8;
        },
        _ => {},
    }
    ary
}

