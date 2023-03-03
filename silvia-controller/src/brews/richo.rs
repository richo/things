use crate::{Brew, Silvia, Conclusion, StopReason};

/// richo's playground brew
pub struct RichoBrew;

impl Brew for RichoBrew {
    fn name(&self) -> &'static str {
        "richo"
    }

    fn brew(&self, silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        // Pulse pump on and off for 300/200 3 times
        // TODO(richo) Maybe the counter shouldn't reset for these?
        // TODO(richo) I think this could be even better with opposing ramps, so shorter gaps and
        // longer runs until the it just brews.
        for t in [200, 300, 400] {
            silvia.pump.set_high();
            silvia.until_unless("ramp-up", t, StopReason::Cancel)?;

            silvia.pump.set_low();
            silvia.until_unless("ramp-up", 200, StopReason::Cancel)?;
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        silvia.valve.set_high();
        silvia.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        silvia.until_unless("brew", 35000, StopReason::Either)
    }
}
