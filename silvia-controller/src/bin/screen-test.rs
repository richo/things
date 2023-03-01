#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut silvia = Silvia::new();

    silvia.display_str("Coffee!");

    loop {}
}
