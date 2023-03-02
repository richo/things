#![no_std]
#![no_main]

use silvia_controller::*;

type ActiveBrew = brews::RichoBrew;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();
    silvia.show_brew_name(ActiveBrew::NAME);

    loop {
        silvia.reinit();
        silvia.write_title("ready");

        silvia.delay_ms(2000);
        silvia.brew::<ActiveBrew>();
        silvia.delay_ms(2000);
    }
}
