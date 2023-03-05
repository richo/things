use crate::{Brew, Silvia, Conclusion, StopReason};

/// richo's playground brew
pub struct RichoBrew;

impl Brew for RichoBrew {
    const NAME: &'static str = "richo";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve_on();
        // Pulse pump on and off for 300/200 3 times
        // TODO(richo) Maybe the counter shouldn't reset for these?
        // TODO(richo) I think this could be even better with opposing ramps, so shorter gaps and
        // longer runs until the it just brews.
        for t in [200, 300, 400] {
            silvia.brew_on();
            silvia.until_unless("ramp-up", t, StopReason::Cancel)?;

            silvia.brew_off();
            silvia.until_unless("ramp-up", 200, StopReason::Cancel)?;
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        silvia.valve_on();
        silvia.brew_on();

        // We'll run the pump for 35s or until someone stops us
        silvia.until_unless("brew", 35000, StopReason::Either)
    }
}
