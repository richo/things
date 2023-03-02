#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    let time = silvia.millis();
    let op = Conclusion::interrupted("testing!", time).unwrap_err();
    silvia.report(op);

    loop {
        let time = silvia.millis();
        silvia.write_time(time);
        spin_wait();
    }
}
