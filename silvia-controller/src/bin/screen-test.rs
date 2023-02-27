#![no_std]
#![no_main]
use hd44780_driver;

use silvia_controller::*;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);



    lcd.reset(&mut delay);
    lcd.clear(&mut delay);
    lcd.write_str("Hello, world!", &mut delay);


    loop {}
}
