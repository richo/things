#![no_std]
#![no_main]

use silvia_controller::*;

fn mainloop(silvia: &mut Silvia) -> Option<Conclusion> {
        if silvia.brew.poll() {
            silvia.log("brew switch");
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
        } else if silvia.nextcancel.poll() {
            silvia.log("next/cancel switch");
            discard(silvia.next_brew());
        }

        None
}

const RESET_INTERVAL: u32 = 5000;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();
    // Show the welcome screen for a second.
    discard(silvia.show_welcome());
    silvia.delay_ms(800);

    //TODO(richo) do something clever to show version if released.
    discard(silvia.show_current_git_hash());
    silvia.delay_ms(3000);

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
            }
            Some(Conclusion::Ok(time)) => {
                silvia.last = Some(time);
                discard(silvia.reset_display());
                // We ran to conclusion, ask if the user wants to flush?
                if silvia.until_unless("flush?", 5000, StopReason::Brew, Count::None).is_err() {
                    silvia.flush();
                }
            }
        }
    }
}
