#![no_std]
#![no_main]
use silvia_controller::*;

use panic_halt as _;
use arduino_hal::port::{mode, Pin};


#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();
    silvia.show_brew_name("richo shot").unwrap();

    let time = silvia.millis();
    let op = Conclusion::interrupted("testing!", time).unwrap_err();
    silvia.report(op);

    loop {
        let time = silvia.millis();
        silvia.write_time(time);
        spin_wait();

        let _ = ufmt::uwriteln!(silvia.serial(), "{}",  crate::brew_switch());
    }
}
