#![no_std]
#![no_main]
use hd44780_driver;

use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut devices = Devices::new();

    loop {
        devices.brew_on();
        devices.delay_ms(500);
        devices.brew_off();
        devices.delay_ms(500);
    }
}
