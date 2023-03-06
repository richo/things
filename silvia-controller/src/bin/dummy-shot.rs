#![no_std]
#![no_main]

use silvia_controller::*;


#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    loop {
        discard(silvia.reinit());
        discard(silvia.write_title("ready"));

        silvia.delay_ms(2000);
        discard(silvia.next_brew());
        silvia.delay_ms(2000);



        silvia.delay_ms(2000);
        let _ = silvia.do_brew();
        silvia.delay_ms(2000);
    }
}
