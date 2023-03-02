use crate::{Brew, Silvia, Conclusion, StopReason};

/// richo's playground brew
pub struct RichoBrew;

impl Brew for RichoBrew {
    const NAME: &'static str = "richo";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        // Pulse pump on and off for 300/200 3 times
        // TODO(richo) Maybe the counter shouldn't reset for these?
        for t in [200, 300, 400] {
            silvia.pump.set_high();
            silvia.until_unless("ramp-up", t, StopReason::Brew)?;

            silvia.pump.set_low();
            silvia.until_unless("ramp-up", 200, StopReason::Brew)?;
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        silvia.valve.set_high();
        silvia.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        silvia.until_unless("brew", 35000, StopReason::Either)
    }
}
