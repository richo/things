#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    // silvia.write_title("Coffee!");

    loop {
        let time = silvia.millis();
        silvia.write_time(time);
        spin_wait();
    }
}
