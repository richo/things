use crate::{Brew, Silvia, until_unless, Conclusion};

/// A straight brew, this is basically the simplest possible profile. Pump turns on for 35s, then
/// turns off.
struct StraightBrew;

impl Brew for StraightBrew {
    const LOGLINE: &'static str = "straight";
    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        silvia.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        until_unless(3500, || silvia.brew.is_low(), |_| {})
    }
}
