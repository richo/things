#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal::prelude::*;
use arduino_hal::hal::port::{Pin, PB2, PB3, PC4, PC5};
use arduino_hal::hal::port::mode::{Input, Output, PullUp};

type PUMP = Pin<Output, PB2>;
type VALVE = Pin<Output, PB3>;

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

    let brew =  pins.a4.into_pull_up_input();
    let backflush =  pins.a5.into_pull_up_input();

    let mut pump = pins.d10.into_output();
    let mut valve = pins.d11.into_output();
    init_pins(&mut pump, &mut valve);

    loop {
        led.toggle();
        // TODO(richo) Migrate to doing an interrupt thing here instead of shitty histerisis
        if brew.is_low() {
            ufmt::uwriteln!(&mut serial, "brew switch").void_unwrap();

            ufmt::uwriteln!(&mut serial, "starting infuse").void_unwrap();
            if let Conclusion::Stopped = run_infuse(&mut pump, &mut valve, &brew) {
                continue
            }
            ufmt::uwriteln!(&mut serial, "infusion finished").void_unwrap();
            ufmt::uwriteln!(&mut serial, "starting brew").void_unwrap();
            let res = run_brew(&mut pump, &mut valve, &brew);
            match res {
                Conclusion::Finished => {
                    ufmt::uwriteln!(&mut serial, "brew finished").void_unwrap();
                },
                Conclusion::Stopped => {
                    ufmt::uwriteln!(&mut serial, "brew interupted").void_unwrap();
                },
            }
            while brew.is_low() {
                spin_wait();
            }
        } else if backflush.is_low() {
            ufmt::uwriteln!(&mut serial, "backflush switch").void_unwrap();
            run_backflush(&mut pump, &mut valve, &backflush);
            while backflush.is_low() {
                spin_wait();
            }
            ufmt::uwriteln!(&mut serial, "backflush finished").void_unwrap();
        }

        // Set them low on every iteration just to be safe.
        init_pins(&mut pump, &mut valve);
        spin_wait();
    }
}

fn init_pins(pump: &mut Pin<Output, PB2>, valve: &mut Pin<Output, PB3>) {
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

fn run_infuse(pump: &mut PUMP, valve: &mut VALVE, switch: &Pin<Input<PullUp>, PC4>) -> Conclusion {
    // Infuse the puck by closing the solenoid and running the pump, but do not open the valve when
    // finished.
    valve.set_high();
    pump.set_high();
    if let Conclusion::Stopped = until_unless(INFUSE_MILLIS, || switch.is_low()) {
        pump.set_low();
        valve.set_low();
        return Conclusion::Stopped;
    }
    pump.set_low();
    until_unless(INFUSE_WAIT_MILLIS, || switch.is_low())
}

enum Conclusion {
    Finished,
    Stopped,
}

/// Confirm the solenoid is closed, then run the brew pump for some configurable number of millies,
/// then turn off the pump and the solenoid
fn run_brew(pump: &mut PUMP, valve: &mut VALVE, switch: &Pin<Input<PullUp>, PC4>) -> Conclusion {
    // Infuse leaves the valve closed, but we'll double check
    valve.set_high();
    pump.set_high();

    // We'll run the pump for 35s or until someone stops us
    // TODO(richo) Again holy shit this should be an interrupt thing
    until_unless(3500, || switch.is_low())
}


const RESOLUTION: u16 = 20;
fn until_unless<F>(millis: u16, unless: F) -> Conclusion
where F: Fn() -> bool {
    for i in 0..(millis / RESOLUTION) {
        if unless() {
            // Wait until the condition clears
            while unless() {
                arduino_hal::delay_ms(RESOLUTION);
            }
            return Conclusion::Stopped;
        }
        arduino_hal::delay_ms(RESOLUTION);
    }
    Conclusion::Finished
}

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 9000;

fn run_backflush(pump: &mut PUMP, valve: &mut VALVE, switch: &Pin<Input<PullUp>, PC5>) {
    for _ in 0..BACKFLUSH_REPEATS {
        valve.set_high();
        pump.set_high();
        let res = until_unless(BACKFLUSH_ON_MILLIS, || switch.is_low());
        pump.set_low();
        valve.set_low();
        if let Conclusion::Finished = res {
            until_unless(BACKFLUSH_PAUSE_MILLIS, || switch.is_low());
        }
    }
}
