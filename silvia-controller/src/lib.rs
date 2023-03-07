#![no_std]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
pub use arduino_hal::prelude::*;
#[allow(unused_imports)]
use arduino_hal::hal::port::{Pin, PB2, PB3, PB4, PD6, PD5, PD4, PD3, PC4, PC5, PD1, PD0, PB0, PB1, PB5};
use arduino_hal::hal::port::mode::Output;

#[cfg(feature = "logging")]
use arduino_hal::hal::pac::USART0;

use hd44780_driver::{
    HD44780,
    bus::FourBitBus,
    error::Error as DisplayError,
};

pub mod millis;
pub mod brews;
pub mod switches;

pub enum Switch {
    Brew,
    NextCancel,
}

pub enum Row {
    First,
    Second,
}

/// How long we pause to give buttons a chance to come up, or after user interactions.
pub const BUTTON_DELAY: u16 = 300;

impl Into<u8> for Row {
    fn into(self) -> u8 {
        match self {
            Row::First => 0,
            Row::Second => 40
        }
    }
}

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

    led: Pin<Output, PB5>,

    current: brews::BrewContainer,
    pub last: Option<u32>,
}

impl Silvia {
    pub fn new() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();
        let pins = arduino_hal::pins!(dp);

        unsafe {
            // SAFETY: Interrupts are not yet enabled, so we can safely: Move the pins required for
            // the switches into a global only used by the interrupt, and then actually configure
            // the interrupts.
            let brew =  pins.a4.into_pull_up_input();
            let nextcancel =  pins.a5.into_pull_up_input();
            switches::init(dp.EXINT, brew, nextcancel);

            // Configure the interrupts for our timers:
            millis::init(dp.TC0);

        avr_device::interrupt::enable()
        };

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
            led,

            current,
            last: None,
        };
        discard(res.reinit());
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
        let b = self.current;
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

    pub fn led(&mut self) -> &mut Pin<Output, PB5> {
        &mut self.led
    }

    /// Move to the next brew, erasing the currently displayed time or extra.
    pub fn next_brew(&mut self) -> Result<(), DisplayError> {
        self.current = self.current.next();
        // Clear the old shot timer
        self.last = None;
        self.show_current_brew_name()
    }

    /// 4 characters worth of extra "scratch space"
    /// This will be overwritten by the elapsed time when the brew ends
    pub fn write_extra(&mut self, extra: &[u8; 4]) -> Result<(), DisplayError> {
        let mut bytes = pad_str(self.current.name(), None);
        bytes[12..].copy_from_slice(extra);
        self.write_buf(&bytes, Row::Second)
    }

    /// Write the current brew name to screen, including any extra or time.
    pub fn show_current_brew_name(&mut self) -> Result<(), DisplayError> {
        let bytes = pad_str(self.current.name(), self.last);
        self.write_buf(&bytes, Row::Second)
    }

    /// Write the contents of `buf` to the display, on the selected Row.
    fn write_buf(&mut self, buf: &[u8; 16], row: Row) -> Result<(), DisplayError> {
        self.lcd.set_cursor_pos(row.into(), &mut self.delay)?;
        self.lcd.write_bytes(buf, &mut self.delay)
    }

    /// Write a time into the upper right hand corner, this is the active counter and is where a
    /// user expects to see the time of the current operation.
    pub fn write_time(&mut self, time: u32) -> Result<(), DisplayError> {
        let mut buf = [0; 4];
        let pos = 12;
        format_time(&mut buf, time);

        self.lcd.set_cursor_pos(pos, &mut self.delay)?;
        self.lcd.write_bytes(&buf, &mut self.delay)
    }

    /// Write a title to the top part of the screen, erasing any time that is currently displayed.
    pub fn write_title(&mut self, title: &str) -> Result<(), DisplayError> {
        self.lcd.set_cursor_pos(0, &mut self.delay)?;
        let bytes = pad_str(title, None);
        self.lcd.write_bytes(&bytes, &mut self.delay)
    }

    pub fn display<'a>(&'a self) -> &'a Display {
        &self.lcd
    }

    #[inline(always)]
    pub fn delay_ms(&self, time: u16) {
        arduino_hal::delay_ms(time)
    }

    fn unless(&mut self, reason: StopReason) -> bool {
        match reason {
            StopReason::Brew => switches::brew(),
            StopReason::Cancel => switches::nextcancel(),
            StopReason::Either => switches::brew() || switches::nextcancel(),
            StopReason::None => false,
        }
    }

    pub fn until_unless(&mut self, op: &'static str, millis: u16, stop: StopReason, count: Count) -> Conclusion {
        discard(self.write_title(op));

        let start = millis::millis();
        let target = start + millis as u32;
        while millis::millis() < target {
            if self.unless(stop) {
                // Wait until the condition clears
                while self.unless(stop) {
                    arduino_hal::delay_ms(RESOLUTION);
                }
                return Conclusion::interrupted(millis::millis() - start);
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
pub fn discard(_: Result<(), DisplayError>) {
}


pub fn spin_wait() {
    arduino_hal::delay_ms(100);
}

pub type Operation = u32;

pub trait OperationExt: Sized {
    fn interrupted(time: u32) -> Self;

    fn finished(time: u32) -> Self;

    fn done() -> Self;
}

impl OperationExt for Result<Operation, Operation> {
    fn interrupted(time: u32) -> Self {
        Err(time)
    }

    fn finished(time: u32) -> Self {
        Ok(time)
    }

    // This is jank for now, we'll probably make this clearer in the type system but for now 0 is a
    // magic value
    fn done() -> Self {
        Ok(0)
    }
}


/// Represents how a logical operation ended.
///
/// Ok(time) represents the operation completing, either with Some(time) representing how long the
/// operation took, or None for "complete" in a context where that time being meaningless.
///
/// Err(time) represents a user intervention.
pub type Conclusion = Result<Operation, Operation>;

// TODO(richo) pull this out and ditch ufmt entirely
const RESOLUTION: u16 = 100;
/// Pad a string out to 16 characters, in order to make it consume a full line
/// If `time` is Some that time will be inserted on the right side, and in the specialcase that
/// it's 0 the string 'done' will be used instead.
fn pad_str(msg: &str, time: Option<u32>) -> [u8; 16] {
    let mut ary = [b' '; 16];
    ary[0..msg.len()].copy_from_slice(msg.as_bytes());
    if let Some(t) = time {
        if t > 0 {
            format_time(&mut ary[12..], t);
        } else {
            ary[12..].copy_from_slice(b"done");
        }
    }
    ary
}

fn format_time(buf: &mut [u8], time: u32) {
    let zero = b'0';
    let secs = time / 1000;
    let tenths = (time % 1000) / 100;

    buf[0] = b' ';
    if secs > 10 {
        buf[0] = zero + (secs / 10) as u8;
    }

    buf[1] = zero + (secs % 10) as u8;
    buf[2] = b'.';
    buf[3] = zero + tenths as u8;
}
