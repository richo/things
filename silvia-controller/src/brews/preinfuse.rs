use crate::{Brew, Silvia, Conclusion, StopReason};

/// This is mostly a reimplementation of what the auber does. 1.2s on, 2.5 off, and then a 25s pull. The 3way valve is opened between the preinfuse and brew steps.
pub struct PreInfuse;

const INFUSE_MILLIS: u16 = 1200;
const INFUSE_WAIT_MILLIS: u16 = 2500;
const BREW_MILLIS: u16 = 25000;

impl Brew for PreInfuse {
    const NAME: &'static str = "preinfuse";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        silvia.pump.set_high();

        // Infuse the puck by closing the solenoid and running the pump
        silvia.until_unless("infuse", INFUSE_MILLIS, StopReason::Brew)?;

        silvia.valve.set_low();
        silvia.pump.set_low();

        silvia.until_unless("wait", INFUSE_WAIT_MILLIS, StopReason::Brew)?;

        silvia.valve.set_high();
        silvia.pump.set_high();

        silvia.until_unless("brew", BREW_MILLIS, StopReason::Brew)
    }
}
