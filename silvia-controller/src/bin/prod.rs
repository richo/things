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
                Conclusion::Ok(()) => {
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
            silvia.next_brew();
        }

        None
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    loop {
        silvia.reinit();
        silvia.write_title("ready");
        // Lock out the machine for a couple of seconds, so that pressing a button right as it
        // stops doesn't start a new one.

        match mainloop(&mut silvia) {
            None => {
                // Nothing happed, busywait and then continue
                spin_wait();
            },
            Some(Conclusion::Err(Operation { name, time })) => {
                // Someone pushed a button, wait for no buttons to be pressed and then continue
                silvia.reset_display();
                silvia.write_time(time);

                while silvia.brew_switch() || silvia.nextcancel_switch() {
                    spin_wait();
                }
                let _ = silvia.until_unless("standby", 1500, StopReason::None, Count::None);
            }
            Some(Conclusion::Ok(())) => {
                silvia.reset_display();
                // We ran to conclusion, do nothing.
                let _ = silvia.until_unless("standby", 1500, StopReason::None, Count::None);
            }
        }
    }
}
