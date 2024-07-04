#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;
mod debounced;
mod millis;

use arduino_hal::hal::port::{Pin, PB2, PB3, PB1, PB5, PC0, PC1, PC2};
use arduino_hal::hal::port::mode::{Input, Output, PullUp};

enum IndicatorState {
    Left,
    Right,
    None,
}

enum BeamState {
    High,
    Low,
}

struct Beams {
    pin: Pin<Output, PB3>,
    state: BeamState,
}

impl Beams {
    fn new(mut pin: Pin<Output, PB3>) -> Self {
        pin.set_low();
        Beams {
            pin,
            state: BeamState::Low,
        }
    }

    fn toggle(&mut self) {
        use BeamState::*;
        match self.state {
            High => {
                self.state = Low;
                self.pin.set_low();
            },
            Low => {
                self.state = High;
                self.pin.set_high();
            }
        }
    }
}

macro_rules! set_indicator (
    ($state:expr, $var:expr, $items:expr) => (
        $var = $state;
        match $var {
            Left => {
                $items.left_indicator.set_high();
                $items.right_indicator.set_low();
            },
            Right => {
                $items.right_indicator.set_high();
                $items.left_indicator.set_low();
            },
            None => {
                $items.right_indicator.set_low();
                $items.left_indicator.set_low();
            }
        }
    )
);

fn mainloop(mut items: Items) -> ! {
   use IndicatorState::*;
   let mut indicator_state = None;

   loop {
       if items.left_button.poll() {
           match indicator_state {
               Left => {
                   set_indicator!(None, indicator_state, items);
               },
               Right => {
                   set_indicator!(Left, indicator_state, items);
               }
               None => {
                   set_indicator!(Left, indicator_state, items);
               }
           };
           while items.left_button.poll() { arduino_hal::delay_ms(100); };
       } else if items.right_button.poll() {
           match indicator_state {
               Right => {
                   set_indicator!(None, indicator_state, items);
               },
               Left => {
                   set_indicator!(Right, indicator_state, items);
               }
               None => {
                   set_indicator!(Right, indicator_state, items);
               }
           };
           while items.right_button.poll() { arduino_hal::delay_ms(100); };
       }

       if items.high_button.poll() {
           items.beams.toggle();
           // wait for the button to be released
           while items.high_button.poll() { arduino_hal::delay_ms(100) }
       }

       arduino_hal::delay_ms(100);
   }
}


fn blinky_test(items: &mut Items) {
    items.led.set_low();
    items.left_indicator.set_low();
    items.right_indicator.set_low();


    // Blink blink
    for _ in 0..2 {
        items.left_indicator.set_high();
        arduino_hal::delay_ms(800);
        items.left_indicator.set_low();
        items.right_indicator.set_high();
        arduino_hal::delay_ms(800);
        items.right_indicator.set_low();
    }

    arduino_hal::delay_ms(500);

    // flash flash flash

    for _ in 0..3 {
        items.left_indicator.set_high();
        items.right_indicator.set_high();
        arduino_hal::delay_ms(500);
        items.left_indicator.set_low();
        items.right_indicator.set_low();
        arduino_hal::delay_ms(200);
    }

    items.led.set_low();
    items.left_indicator.set_low();
    items.right_indicator.set_low();
}

struct Items {
    beams: Beams,
    right_indicator: Pin<Output, PB2>,
    left_indicator: Pin<Output, PB1>,

    high_button: debounced::DebouncedButton<Pin<Input<PullUp>, PC0>>,
    right_button: debounced::DebouncedButton<Pin<Input<PullUp>, PC1>>,
    left_button: debounced::DebouncedButton<Pin<Input<PullUp>, PC2>>,

    led: Pin<Output, PB5>,
}

trait DebouncedShim {
    fn poll(&self) -> bool;
}

impl DebouncedShim for Pin<Input<PullUp>, PC0> {
    fn poll(&self) -> bool {
        self.is_low()
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Fix these numbers since they're wrong.
    let led = pins.d13.into_output();  // B5

    let high_beam = pins.d11.into_output(); // B3
    let right_indicator = pins.d10.into_output(); // B2
    let left_indicator = pins.d9.into_output(); // B1

    let high_button = debounced::DebouncedButton::new(pins.a0.into_pull_up_input()); // C0
    let left_button = debounced::DebouncedButton::new(pins.a2.into_pull_up_input()); // C2
    let right_button = debounced::DebouncedButton::new(pins.a1.into_pull_up_input()); // C1

    let mut items = Items {
        beams: Beams::new(high_beam),
        right_indicator,
        left_indicator,

        high_button,
        right_button,
        left_button,

        led,
    };

    millis::millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable() };

    blinky_test(&mut items);
    mainloop(items);

}
