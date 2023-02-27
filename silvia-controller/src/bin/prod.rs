#![no_std]
#![no_main]

use silvia_controller::*;

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
