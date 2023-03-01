use crate::{Brew, Silvia, until_unless, Conclusion};

/// This is mostly a reimplementation of what the auber does. 1.2s on, 2.5 off, and then a 25s pull. The 3way valve is opened between the preinfuse and brew steps.
pub struct PreInfuse;

const INFUSE_MILLIS: u16 = 1200;
const INFUSE_WAIT_MILLIS: u16 = 2500;
const BREW_MILLIS: u16 = 25000;

impl Brew for PreInfuse {
    const LOGLINE: &'static str = "preinfuse";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        silvia.pump.set_high();

        // Infuse the puck by closing the solenoid and running the pump
        if let Conclusion::Interrupted(i) = until_unless(INFUSE_MILLIS, || silvia.brew.is_low(), |_| {}) {
            return Conclusion::Interrupted(i);

        }
        silvia.valve.set_low();
        silvia.pump.set_low();

        if let Conclusion::Interrupted(i) = until_unless(INFUSE_WAIT_MILLIS, || silvia.brew.is_low(), |_| {}) {
            return Conclusion::Interrupted(i);
        }

        silvia.valve.set_high();
        silvia.pump.set_high();

        until_unless(BREW_MILLIS, || silvia.brew.is_low(), |_| {})
    }
}