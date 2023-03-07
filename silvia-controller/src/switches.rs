use core::sync::atomic::{AtomicBool, Ordering};
use core::mem;

static BREW: AtomicBool = AtomicBool::new(false);
static NEXTCANCEL: AtomicBool = AtomicBool::new(false);

use arduino_hal::hal::port::mode::{Input, PullUp};
use arduino_hal::hal::port::{Pin, PC4, PC5};
use avr_device::atmega328p::EXINT;

type BrewSwitch = Pin<Input<PullUp>, PC4>;
type NextCancelSwitch = Pin<Input<PullUp>, PC5>;

pub type Butts = Option<AtomicBool>;


struct InterruptState {
    brew: BrewSwitch,
    nextcancel: NextCancelSwitch,
}

static mut INTERRUPT_STATE: mem::MaybeUninit<InterruptState> = mem::MaybeUninit::uninit();


#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT1() {
    let state = unsafe {
        // SAFETY: We _know_ that interrupts will only be enabled after the INTERRUPT_STATE is
        // initialized in our init function, so this ISR will never run when INTERRUPT_STATE is
        // uninitialized.
        &mut *INTERRUPT_STATE.as_mut_ptr()
    };

    if state.brew.is_low() {
        BREW.store(true, Ordering::Relaxed);
    }
    if state.nextcancel.is_low() {
        NEXTCANCEL.store(true, Ordering::Relaxed);
    }
}

/// SAFETY: This function can only be called with interrupts disabled.
#[allow(non_snake_case)]
pub unsafe fn init(EXINT: EXINT, brew: BrewSwitch, nextcancel: NextCancelSwitch) {
    INTERRUPT_STATE = mem::MaybeUninit::new(InterruptState {
        brew,
        nextcancel,
    });

    EXINT.pcicr.write(|w| unsafe { w.bits(0b010) });
    EXINT.pcmsk1.write(|w| unsafe {  w.bits(0b110000) });
}

/// Check if the brew switch has been pressed.
///
/// This function resets the internal state, so checking twice in a row will return true at most
/// once for the first press.
pub fn brew() -> bool {
    avr_device::interrupt::free(|_cs| {
        let res = BREW.load(Ordering::Relaxed);
        BREW.store(false, Ordering::Relaxed);
        res
    })
}

/// Check if the nextcancel switch has been pressed.
///
/// This function resets the internal state, so checking twice in a row will return true at most
/// once for the first press.
pub fn nextcancel() -> bool {
    avr_device::interrupt::free(|_cs| {
        let res = NEXTCANCEL.load(Ordering::Relaxed);
        NEXTCANCEL.store(false, Ordering::Relaxed);
        res
    })
}
