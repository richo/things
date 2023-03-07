#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
pub use arduino_hal::prelude::*;
use core::sync::atomic::{AtomicBool, Ordering};
use core::mem;


use arduino_hal::hal::port::mode::{Input, Output, PullUp};
use arduino_hal::hal::port::{Pin, PB2, PB3, PB4, PD6, PD5, PD4, PD3, PC4, PC5, PD1, PD0, PB0, PB1, PB5};
use arduino_hal::hal::pac::USART0;
type Serial = arduino_hal::usart::Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>;

static BREW: AtomicBool = AtomicBool::new(false);
static NEXTCANCEL: AtomicBool = AtomicBool::new(false);

fn brew() -> bool {
    avr_device::interrupt::free(|cs| {
        let res = BREW.load(Ordering::Relaxed);
        BREW.store(false, Ordering::Relaxed);
        res
    })
}

struct InterruptState {
        a4: Pin<Input<PullUp>>,
        a5: Pin<Input<PullUp>>,
}

static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT1() {
    let state = unsafe {
        // SAFETY: We _know_ that interrupts will only be enabled after the LED global was
        // initialized so this ISR will never run when LED is uninitialized.
        &mut *INTERRUPT_STATE.as_mut_ptr()
    };

    if state.a4.is_low() {
        BREW.store(true, Ordering::Relaxed);
    }
    if state.a5.is_low() {
        NEXTCANCEL.store(true, Ordering::Relaxed);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let a4 = pins.a4.into_pull_up_input();
    let a5 = pins.a5.into_pull_up_input();
    let d13 = pins.d13.into_output();

    unsafe {
        // SAFETY: Interrupts are not enabled at this point so we can safely write the global
        // variable here.  A memory barrier afterwards ensures the compiler won't reorder this
        // after any operation that enables interrupts.
        INTERRUPT_STATE = mem::MaybeUninit::new(InterruptState {
            a4: a4.downgrade(),
            a5: a5.downgrade(),
        });
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }


    unsafe { avr_device::interrupt::enable() };



    // let _ = ufmt::uwriteln!(serial, "{}", "oh hello there");
    loop {
        let _ = ufmt::uwriteln!(serial, "{}", brew());
        arduino_hal::delay_ms(1000);
    }
}
