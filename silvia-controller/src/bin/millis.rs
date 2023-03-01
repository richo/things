#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut devices = Devices::new();

    loop {
        let millis = devices.millis();

        devices.delay_ms(1200);

        let new = devices.millis();

        let _ = ufmt::uwriteln!(devices.serial(), "{}",  new - millis);
    }
}
