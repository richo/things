use crate::{Brew, Silvia, Conclusion, StopReason};

/// A straight brew, this is basically the simplest possible profile. Pump turns on for 35s, then
/// turns off.
pub struct StraightBrew;

impl Brew for StraightBrew {
    const NAME: &'static str = "straight";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        silvia.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        silvia.until_unless("brew", 3500, StopReason::Brew)
    }
}
