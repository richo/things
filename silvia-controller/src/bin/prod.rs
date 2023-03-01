#![no_std]
#![no_main]

use silvia_controller::*;

fn mainloop(silvia: &mut Devices) -> Option<Conclusion> {
        // silvia.led().toggle();
        // TODO(richo) Migrate to doing an interrupt thing here instead of shitty histerisis
        if silvia.brew_switch() {
            silvia.log("brew switch");
            // Wait for the switch to come up
            while silvia.brew_switch() {
                spin_wait();
            }
            silvia.log("-> brew");

            silvia.log("starting infuse");
            if let Conclusion::Stopped = silvia.run_infuse() {
                return Some(Conclusion::Stopped);
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
            return Some(res);
        } else if silvia.backflush_switch() {
            silvia.log("backflush switch");
            // Wait for the backflush switch to come up, then start.
            while silvia.backflush_switch() {
                spin_wait();
            }
            silvia.log("-> backflush");
            let res = silvia.run_backflush();
            match res {
                Conclusion::Finished => {
                    silvia.log("backflush finished");
                },
                Conclusion::Stopped => {
                    silvia.log("Backflush interrupted");
                },
            }
            return Some(res);
        }

        None
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Devices::new();

    loop {
        silvia.reinit();
        match mainloop(&mut silvia) {
            None => {
                // Nothing happed, busywait and then continue
                spin_wait();
            },
            Some(Conclusion::Stopped) => {
                // Someone pushed a button, wait for no buttons to be pressed and then continue
                while silvia.brew_switch() || silvia.backflush_switch() {
                    spin_wait();
                }
            }
            Some(Conclusion::Finished) => {
                // We ran to conclusion, do nothing.
            }
        }
    }
}
