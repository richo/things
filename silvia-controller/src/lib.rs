#![no_std]
#![feature(abi_avr_interrupt)]

use core::cmp;
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
use core::sync::atomic::{AtomicBool, Ordering};

pub mod millis;
pub mod brews;
mod formatting;

pub enum Switch {
    Brew,
    BackFlush,
}

static REVERSED: AtomicBool = AtomicBool::new(false);

pub fn is_reversed() -> bool {
    REVERSED.load(Ordering::SeqCst)
}

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let current = REVERSED.load(Ordering::SeqCst);
    REVERSED.store(!current, Ordering::SeqCst);
}

use formatting::BoundDisplay;

pub trait Brew {
    const NAME: &'static str;

    fn run(silvia: &mut Silvia) -> Conclusion {
        Self::log(silvia, "starting brew");
        let res = Self::brew(silvia);
        // Confirm all the relays are closed.
        silvia.valve_off();
        silvia.brew_off();
        res
    }

    fn log(silvia: &mut Silvia, msg: &'static str) {
        let _ = ufmt::uwriteln!(silvia.serial, "{} {}",  Self::NAME, msg);
    }

    /// The main function which interacts with the machine to brew.
    ///
    /// You can safely return without turning off pumps and valves, etc, the frameowrk will take
    /// care of resetting things for you.
    fn brew(silvia: &mut Silvia) -> Conclusion;
}

type Display = HD44780<FourBitBus<Pin<Output, PB4>, Pin<Output, PB3>, Pin<Output, PD6>, Pin<Output, PD5>, Pin<Output, PD4>, Pin<Output, PD3>>>;
type Serial = arduino_hal::usart::Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

pub struct Silvia {
    serial: Serial,
    lcd: Display,
    delay: arduino_hal::Delay,
    pump: Pin<Output, PB1>,
    valve: Pin<Output, PB0>,

    brew: Pin<Input<PullUp>, PC4>,
    backflush: Pin<Input<PullUp>, PC5>,

    led: Pin<Output, PB5>,
}

impl Silvia {
    pub fn new() -> Self {
        let dp = arduino_hal::Peripherals::take().unwrap();

        dp.EXINT.eicra.modify(|_, w| w.isc0().bits(0x02));
        // Enable the INT0 interrupt source.
        dp.EXINT.eimsk.modify(|_, w| w.int0().set_bit());

        millis::millis_init(dp.TC0);
        unsafe { avr_device::interrupt::enable() };

        let pins = arduino_hal::pins!(dp);
        let serial = arduino_hal::default_serial!(dp, pins, 57600);

        // Display
        let rs = pins.d12.into_output();
        let e = pins.d11.into_output();

        let d2 = pins.d2.into_pull_up_input();

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

        let mut res = Silvia {
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

    pub fn brew<B: Brew>(&mut self) -> Conclusion {
        B::brew(self)
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

    pub fn show_brew_name(&mut self, name: &str) -> Result<(), DisplayError> {
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
    fn unless(&mut self) -> bool {
        self.brew.is_low() ||
            self.backflush.is_low()
    }

    fn until_unless(&mut self, op: &'static str, millis: u16, switch: Switch) -> Conclusion {
        // TODO(richo) Show goal time in the lower right?
        self.write_title(op);
        // self.write_goal(millis as u32);

        let start = millis::millis();
        let target = start + millis as u32;
        while millis::millis() < target {
            if self.unless() {
                // Wait until the condition clears
                while self.unless() {
                    arduino_hal::delay_ms(RESOLUTION);
                }
                return Conclusion::time(millis::millis() - start);
            }
            self.write_time(millis::millis() - start);
            arduino_hal::delay_ms(RESOLUTION);
        }
        Ok(())
    }

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

