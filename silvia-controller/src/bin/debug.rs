#![no_std]
#![no_main]
use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let mut devices = Silvia::new();

    loop {
        devices.pump_on();
        devices.valve_on();
        devices.delay_ms(1500);
        devices.pump_off();
        devices.valve_off();
        devices.delay_ms(500);
    }
}
