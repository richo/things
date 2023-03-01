use ufmt::uWrite;
use hd44780_driver::{
    error::Error as DisplayError,
};

/// A display bound to a Delay, which then lets us implement the various write traits to make
/// formatting work.
pub struct BoundDisplay<'a> {
    pub display: &'a mut crate::Display,
    pub delay: &'a mut arduino_hal::Delay,
}

impl<'a> uWrite for BoundDisplay<'a> {
    type Error = DisplayError;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.display.write_str(s, self.delay)
    }
}
