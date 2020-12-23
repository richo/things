#![feature(llvm_asm)]

#![no_std]
#![no_main]

use ruduino::cores::atmega328 as avr_core;
use ruduino::{Pin, DataDirection};

use avr_core::{
    DDRB, PORTB,
    DDRD, PORTD,
};

use avr_core::port::{
    B5 as LED,
    B3 as HIGH_BEAM,
    B2 as RIGHT_INDICATOR,
    B1 as LEFT_INDICATOR,

    C0 as HIGH_BEAM_BUTTON,
    C2 as LEFT_BUTTON,
    C1 as RIGHT_BUTTON,
};



enum IndicatorState {
    Left,
    Right,
    None,
}

enum BeamState {
    High,
    Low,
}

struct Beams {
    state: BeamState,
}

impl Beams {
    fn new() -> Self {
        HIGH_BEAM::set_low();
        Beams {
            state: BeamState::Low,
        }
    }

    fn toggle(&mut self) {
        use BeamState::*;
        match self.state {
            High => {
                self.state = Low;
                HIGH_BEAM::set_low();
            },
            Low => {
                self.state = High;
                HIGH_BEAM::set_high();
            }
        }
    }
}

#[no_mangle]
pub extern fn abort() -> ! {
    loop {}
}


macro_rules! set_indicator (
    ($state:expr, $var:expr) => (
        $var = $state;
        match $var {
            Left => {
                LEFT_INDICATOR::set_high();
                RIGHT_INDICATOR::set_low();
            },
            Right => {
                RIGHT_INDICATOR::set_high();
                LEFT_INDICATOR::set_low();
            },
            None => {
                RIGHT_INDICATOR::set_low();
                LEFT_INDICATOR::set_low();
            }
        }
    )
);

fn init_pins() {
    LED::set_direction(DataDirection::Output);
    RIGHT_INDICATOR::set_direction(DataDirection::Output);
    LEFT_INDICATOR::set_direction(DataDirection::Output);
    HIGH_BEAM::set_direction(DataDirection::Output);

    HIGH_BEAM_BUTTON::set_direction(DataDirection::Input);
    HIGH_BEAM_BUTTON::set_high();
    RIGHT_BUTTON::set_direction(DataDirection::Input);
    RIGHT_BUTTON::set_high();
    LEFT_BUTTON::set_direction(DataDirection::Input);
    LEFT_BUTTON::set_high();
}

fn mainloop() {
   use IndicatorState::*;
   let mut indicator_state = None;
   let mut beams = Beams::new();

   loop {
       if LEFT_BUTTON::is_low() {
           match indicator_state {
               Left => {
                   set_indicator!(None, indicator_state);
               },
               Right => {
                   set_indicator!(Left, indicator_state);
               }
               None => {
                   set_indicator!(Left, indicator_state);
               }
           };
           while LEFT_BUTTON::is_low() { small_delay(); };
       } else if RIGHT_BUTTON::is_low() {
           match indicator_state {
               Right => {
                   set_indicator!(None, indicator_state);
               },
               Left => {
                   set_indicator!(Right, indicator_state);
               }
               None => {
                   set_indicator!(Right, indicator_state);
               }
           };
           while RIGHT_BUTTON::is_low() { small_delay(); };
       }

       if HIGH_BEAM_BUTTON::is_low() {
           beams.toggle();
           // wait for the button to be released
           while HIGH_BEAM_BUTTON::is_low() { really_small_delay() }
       }

       really_small_delay();
   }
}


fn blinky_test() {
    LED::set_low();
    HIGH_BEAM::set_low();
    LEFT_INDICATOR::set_low();
    RIGHT_INDICATOR::set_low();


    // Blink blink
    for _ in 0..2 {
        LEFT_INDICATOR::set_high();
        small_delay();
        LEFT_INDICATOR::set_low();
        RIGHT_INDICATOR::set_high();
        small_delay();
        RIGHT_INDICATOR::set_low();
    }

    delay(100000);

    // flash flash flash

    for _ in 0..3 {
        LEFT_INDICATOR::set_high();
        RIGHT_INDICATOR::set_high();
        delay(150000);
        LEFT_INDICATOR::set_low();
        RIGHT_INDICATOR::set_low();
        delay(150000);
    }

    LED::set_low();
    LEFT_INDICATOR::set_low();
    RIGHT_INDICATOR::set_low();
}

#[no_mangle]
pub extern fn main() {
    init_pins();
    blinky_test();
    mainloop();
}

fn delay(len: u64) {
    for _ in 0..len {
        unsafe { llvm_asm!("" :::: "volatile")}
    }
}

/// A small busy loop.
fn small_delay() {
    delay(400000);
}

/// A realsmall busy loop.
fn really_small_delay() {
    for _ in 0..4000 {
        unsafe { llvm_asm!("" :::: "volatile")}
    }
}
