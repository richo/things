use crate::{Brew, Silvia, Conclusion, StopReason, Count};

/// Attempt to trigger the screen issues on purpose for debugging
pub struct Repro;

/// Backflush the machine.
const REPRO_REPEATS: u16 = 3;
const REPRO_ON_MILLIS: u16 = 800;
const REPRO_PAUSE_MILLIS: u16 = 3000;

impl Brew for Repro {
    const NAME: &'static str = "backflush";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        for _ in 0..REPRO_REPEATS {
            silvia.valve_on();
            silvia.pump_on();
            silvia.until_unless("flush", REPRO_ON_MILLIS, StopReason::Cancel, Count::DownFrom(REPRO_ON_MILLIS as u32))?;

            silvia.pump_off();
            silvia.valve_off();
            silvia.until_unless("wait", REPRO_PAUSE_MILLIS, StopReason::Cancel, Count::DownFrom(REPRO_PAUSE_MILLIS as u32))?;
        }
        Ok(())
    }
}
