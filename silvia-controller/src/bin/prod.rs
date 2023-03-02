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
        silvia.delay_ms(2000);
        let res = silvia.brew::<ActiveBrew>();
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
            }
            Some(Conclusion::Ok(())) => {
                // We ran to conclusion, do nothing.
            }
        }
    }
}
