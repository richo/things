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
    // B3 as HIGH_BEAM,
    // B2 as RIGHT_INDICATOR,
    // B1 as LEFT_INDICATOR,
    B3 as PUMP,
    B2 as VALVE,

    // C0 as HIGH_BEAM_BUTTON,
    // C2 as LEFT_BUTTON,
    // C1 as RIGHT_BUTTON,
    C0 as BREW_BUTTON,
    C1 as BACKFLUSH_BUTTON,
};

#[no_mangle]
pub extern fn abort() -> ! {
    loop {}
}


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
   loop {
       if BACKFLUSH_BUTTON::is_low() {
       } else if BREW_BUTTON::is_low() {
       }
       really_small_delay();
   }
}


fn blinky_test() {
    LED::set_low();
    HIGH_BEAM::set_low();
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
