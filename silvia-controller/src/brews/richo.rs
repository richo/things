use crate::{Brew, Silvia, Conclusion, StopReason, Count};

/// richo's playground brew
pub struct RichoBrew;

impl Brew for RichoBrew {
    const NAME: &'static str = "richo";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve_on();
        // Pulse pump on and off for 300/200 3 times
        // TODO(richo) I think this could be even better with opposing ramps, so shorter gaps and
        // longer runs until the it just brews.
        //
        // This is a bit of a hack but for prettiness sake we're going to figure out the total time
        // we're going to count down and then manually bring it down for each call
        let mut td = 200 + 200 + 300 + 200 + 400 + 200;
        for t in [200, 300, 400] {
            silvia.pump_on();
            silvia.until_unless("ramp-up", t, StopReason::Cancel, Count::DownFrom(td))?;
            td -= t as u32;

            silvia.pump_off();
            silvia.until_unless("ramp-up", 200, StopReason::Cancel, Count::DownFrom(td))?;
            td -= 200;
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        silvia.valve_on();
        silvia.pump_on();

        // We'll run the pump for 35s or until someone stops us
        silvia.until_unless("brew", 35000, StopReason::Either, Count::Up)
    }
}
