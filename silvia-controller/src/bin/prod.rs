#![no_std]
#![no_main]

use silvia_controller::*;

fn mainloop(silvia: &mut Silvia) -> Option<Conclusion> {
        if silvia.brew_switch() {
            silvia.log("brew switch");
            // Wait for the switch to come up
            while silvia.brew_switch() {
                spin_wait();
            }
            silvia.log("-> brew");

            let res = silvia.do_brew();
            match res {
                Conclusion::Ok(_) => {
                    silvia.log("brew finished");
                },
                Conclusion::Err(_) => {
                    silvia.log("brew interupted");
                },
            }
            return Some(res);
        } else if silvia.nextcancel_switch() {
            silvia.log("next/cancel switch");
            // Wait for the backflush switch to come up, then start.
            while silvia.nextcancel_switch() {
                spin_wait();
            }
            discard(silvia.next_brew());
        }

        None
}

const RESET_INTERVAL: u32 = 5000;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    let mut last_reset = millis::millis();
    loop {
        // Reset the display every 5s since it's glitchy
        let now = millis::millis();
        if now - last_reset > RESET_INTERVAL {
            last_reset = now;
            discard(silvia.reset_display());
        }
        discard(silvia.reinit());
        discard(silvia.write_title("ready"));
        // Lock out the machine for a couple of seconds, so that pressing a button right as it
        // stops doesn't start a new one.

        match mainloop(&mut silvia) {
            None => {
                // Nothing happed, busywait and then continue
                spin_wait();
            },
            Some(Conclusion::Err(time)) => {
                // Someone pushed a button, wait for no buttons to be pressed and then continue
                silvia.last = Some(time);
                discard(silvia.reset_display());

                while silvia.brew_switch() || silvia.nextcancel_switch() {
                    spin_wait();
                }
            }
            Some(Conclusion::Ok(time)) => {
                silvia.last = Some(time);
                discard(silvia.reset_display());
                // We ran to conclusion, do nothing.
                let _ = silvia.until_unless("standby", 1500, StopReason::None, Count::None);
            }
        }
    }
}
