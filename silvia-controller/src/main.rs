#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
use arduino_hal::prelude::*;
use arduino_hal::hal::port::{Pin, PB1, PB2, PB3, PB5, PC4, PC5};
use arduino_hal::hal::port::mode::{Input, Output, PullUp, Floating};

type PUMP = Pin<Output, PC4>;
type VALVE = Pin<Output, PC5>;

use core::sync::atomic::{AtomicBool, Ordering};

static BREW: AtomicBool = AtomicBool::new(false);
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() { // D10
        BREW.store(true, Ordering::SeqCst);
}

static BACKFLUSH: AtomicBool = AtomicBool::new(false);
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT1() { // D9
        BACKFLUSH.store(true, Ordering::SeqCst);
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);


    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut led = pins.d13.into_output();

    let brew =  pins.d9.into_pull_up_input();
    let backflush =  pins.d10.into_floating_input();

    let mut pump = pins.a4.into_output();
    let mut valve = pins.a5.into_output();
    init_pins(&mut pump, &mut valve);
    dp.EXINT.eicra.modify(|_, w| w.isc0().bits(0x01));
    dp.EXINT.eimsk.modify(|_, w| w.int0().set_bit());
    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0xff) });
    unsafe { avr_device::interrupt::enable() };
    // core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    loop {
        led.toggle();
        let b = BREW.load(Ordering::SeqCst);
        let ba = BACKFLUSH.load(Ordering::SeqCst);
        ufmt::uwriteln!(&mut serial, "{:?} {:?}", b, ba).void_unwrap();
        spin_wait();
    }

    loop {
        led.toggle();
        let b = BREW.load(Ordering::SeqCst);
        let ba = BACKFLUSH.load(Ordering::SeqCst);
        ufmt::uwriteln!(&mut serial, "{:?} {:?}", b, ba).void_unwrap();
        // TODO(richo) Migrate to doing an interrupt thing here instead of shitty histerisis
        if brew.is_low() {
            ufmt::uwriteln!(&mut serial, "brew switch").void_unwrap();

            ufmt::uwriteln!(&mut serial, "starting infuse").void_unwrap();
            run_infuse(&mut pump, &mut valve);
            ufmt::uwriteln!(&mut serial, "infusion finished").void_unwrap();
            ufmt::uwriteln!(&mut serial, "starting brew").void_unwrap();
            let res = run_brew(&mut pump, &mut valve, &brew);
            match res {
                BrewConclusion::Finished => {
                    ufmt::uwriteln!(&mut serial, "brew finished").void_unwrap();
                },
                BrewConclusion::Stopped => {
                    ufmt::uwriteln!(&mut serial, "brew interupted").void_unwrap();
                },
            }
            while brew.is_low() {
                spin_wait();
            }
        } else if backflush.is_low() {
            ufmt::uwriteln!(&mut serial, "backflush switch").void_unwrap();
            run_backflush(&mut pump, &mut valve);
            while backflush.is_low() {
                spin_wait();
            }
            ufmt::uwriteln!(&mut serial, "backflush finished").void_unwrap();
        }

        spin_wait();
        // Set them low on every iteration just to be safe.
        init_pins(&mut pump, &mut valve);
    }
}

fn init_pins(pump: &mut Pin<Output, PC4>, valve: &mut Pin<Output, PC5>) {
    pump.set_low();
    valve.set_low();
}

fn spin_wait() {
    arduino_hal::delay_ms(100);
}

/// Turn on the pump and solenoid, wait some configurable number of millis, turn off the pump, wait
/// some configurable number of millis, without opening the 3 way valve.
const INFUSE_MILLIS: u16 = 2000;
const INFUSE_WAIT_MILLIS: u16 = 2500;

fn run_infuse(pump: &mut PUMP, valve: &mut VALVE) {
    return;
    // Infuse the puck by closing the solenoid and running the pump, but do not open the valve when
    // finished.
    valve.set_high();
    pump.set_high();
    arduino_hal::delay_ms(INFUSE_MILLIS);
    pump.set_low();
    arduino_hal::delay_ms(INFUSE_WAIT_MILLIS);
}

enum BrewConclusion {
    Finished,
    Stopped,
}

/// Confirm the solenoid is closed, then run the brew pump for some configurable number of millies,
/// then turn off the pump and the solenoid
fn run_brew(pump: &mut PUMP, valve: &mut VALVE, switch: &Pin<Input<PullUp>, PB1>) -> BrewConclusion {
    return BrewConclusion::Finished;
    // Infuse leaves the valve closed, but we'll double check
    valve.set_high();
    pump.set_high();

    // We'll run the pump for 35s or until someone stops us
    // TODO(richo) Again holy shit this should be an interrupt thing
    // Whatever this doesn't actually need to be scientific
    const RESOLUTION: u16 = 20;
    for _ in 0..(35000 / RESOLUTION) {
        if switch.is_low() {
            return BrewConclusion::Stopped;
        }
        arduino_hal::delay_ms(RESOLUTION);
    }
    return BrewConclusion::Finished;
}

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 9000;

fn run_backflush(pump: &mut PUMP, valve: &mut VALVE) {
    return;
    for _ in 0..BACKFLUSH_REPEATS {
        valve.set_high();
        pump.set_high();
        arduino_hal::delay_ms(BACKFLUSH_ON_MILLIS);
        pump.set_low();
        valve.set_low();
        arduino_hal::delay_ms(BACKFLUSH_PAUSE_MILLIS);
    }
}
