#![no_std]
#![no_main]

use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Devices::new();

    loop {
        silvia.led().toggle();
        // TODO(richo) Migrate to doing an interrupt thing here instead of shitty histerisis
        if silvia.brew_switch() {
            silvia.log("brew switch");

            silvia.log("starting infuse");
            if let Conclusion::Stopped = silvia.run_infuse() {
                continue
            }
            silvia.log("infusion finished");
            silvia.log("starting brew");
            let res = silvia.run_brew();
            match res {
                Conclusion::Finished => {
                    silvia.log("brew finished");
                },
                Conclusion::Stopped => {
                    silvia.log("brew interupted");
                },
            }
            while silvia.brew_switch() {
                spin_wait();
            }
        } else if silvia.backflush_switch() {
            silvia.log("backflush switch");
            silvia.run_backflush();
            while silvia.backflush_switch() {
                spin_wait();
            }
            silvia.log("backflush finished");
        }

        // Set them low on every iteration just to be safe.
        silvia.reinit();
        spin_wait();
    }
}
