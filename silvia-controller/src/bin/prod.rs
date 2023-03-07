#![no_std]
#![no_main]

use silvia_controller::*;

fn mainloop(silvia: &mut Silvia) -> Option<Conclusion> {
        if switches::brew() {
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
        } else if switches::nextcancel() {
            silvia.log("next/cancel switch");
            discard(silvia.next_brew());
        }

        None
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    loop {
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
                // We ran to conclusion, do nothing.
                let _ = silvia.until_unless("standby", 1500, StopReason::None, Count::None);
            }
        }
    }
}
