#![no_std]
#![no_main]

use silvia_controller::*;

type ActiveBrew = brews::RichoBrew;

fn mainloop(silvia: &mut Silvia) -> Option<Conclusion> {
        if silvia.brew_switch() {
            silvia.log("brew switch");
            // Wait for the switch to come up
            while silvia.brew_switch() {
                spin_wait();
            }
            silvia.log("-> brew");

            let res = silvia.brew::<ActiveBrew>();
            match res {
                Conclusion::Ok(()) => {
                    silvia.log("brew finished");
                },
                Conclusion::Err(_) => {
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
            // Specialcase backflushing and show that for a sec
            silvia.show_brew_name(brews::BackFlush::NAME);

            silvia.log("-> backflush");
            let res = silvia.brew::<brews::BackFlush>();
            match res {
                Conclusion::Ok(()) => {
                    silvia.log("backflush finished");
                },
                Conclusion::Err(_) => {
                    silvia.log("Backflush interrupted");
                },
            }
            silvia.show_brew_name(ActiveBrew::NAME);
            return Some(res);
        }

        None
}

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();
    silvia.show_brew_name(ActiveBrew::NAME);

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
            Some(Conclusion::Err(_)) => {
                // Someone pushed a button, wait for no buttons to be pressed and then continue
                while silvia.brew_switch() || silvia.backflush_switch() {
                    spin_wait();
                }
                let _ = silvia.until_unless("standby", 2000, StopReason::None);
            }
            Some(Conclusion::Ok(())) => {
                // We ran to conclusion, do nothing.
                let _ = silvia.until_unless("standby", 2000, StopReason::None);
            }
        }
    }
}
