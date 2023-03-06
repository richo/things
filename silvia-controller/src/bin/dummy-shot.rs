#![no_std]
#![no_main]

use silvia_controller::*;


#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    loop {
        silvia.reinit();
        silvia.write_title("ready");

        silvia.delay_ms(2000);
        silvia.next_brew();
        silvia.delay_ms(2000);



        silvia.delay_ms(2000);
        silvia.do_brew();
        silvia.delay_ms(2000);
    }
}
