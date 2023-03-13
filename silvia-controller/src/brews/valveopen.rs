use crate::{Brew, Silvia, Conclusion, StopReason, Count};

/// Test brew to see if you can preinfuse by opening the valve and waiting on the boiler pressure
/// to wet the puck.
pub struct ValveOpen;

impl Brew for ValveOpen {
    const NAME: &'static str = "valveopen";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve_on();
        // For now this is just a test of that idea, so chill out for 10s with the valve open then
        // stop.
        silvia.until_unless("brew", 10000, StopReason::Either, Count::DownFrom(10000))
    }
}
