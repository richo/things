#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();
    // silvia.show_brew_name("richo shot").unwrap();

    // let time = silvia.millis();
    // let op = Conclusion::interrupted("testing!", time);
    // if let Err(op) = op {
    //     silvia.report(op);
    // }

    silvia.last = Some(6666);
    discard(silvia.next_brew());

    silvia.delay_ms(5000);



    loop {
        let time = millis::millis();
        discard(silvia.write_time(time));
        spin_wait();
    }
}
