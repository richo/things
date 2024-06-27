use crate::millis;
use arduino_hal::hal::port::{Pin, PC0, PC1, PC2, PC4, PC5};
use arduino_hal::hal::port::mode::{Input, PullUp};

pub trait Poll {
    fn poll(&self) -> bool;
}

pub struct DebouncedButton<P: Poll> {
    pin: P,
    last: u32,
}

const DEBOUNCE_THRESHOLD: u32 = 200;

impl<P: Poll> DebouncedButton<P> {
    pub fn new(switch: P) -> Self {
        Self {
            pin: switch,
            last: 0,
        }
    }

    pub fn poll(&mut self) -> bool {
        let now = millis::millis();
        if now < self.last + DEBOUNCE_THRESHOLD {
            return false;
        }

        if self.pin.poll() {
            self.last = now;
            return true;
        }
        false
    }
}

macro_rules! impl_poll (
    ($($id:ty),*) => (

        $(
            impl Poll for Pin<Input<PullUp>, $id> {
                fn poll(&self) -> bool {
                    self.is_low()
                }
            }
        )*

    )
);

impl_poll!(PC0, PC1, PC2);
