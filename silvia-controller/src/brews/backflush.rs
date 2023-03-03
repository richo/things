use crate::{Brew, Silvia, Conclusion, StopReason};

/// Backflush the machine
pub struct BackFlush;

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 7000;

impl Brew for BackFlush {
    fn name(&self) -> &'static str {
        "backflush"
    }

    fn brew(&self, silvia: &mut Silvia) -> Conclusion {
        for _ in 0..BACKFLUSH_REPEATS {
            silvia.valve.set_high();
            silvia.pump.set_high();
            silvia.until_unless("flush", BACKFLUSH_ON_MILLIS, StopReason::Cancel)?;

            silvia.pump.set_low();
            silvia.valve.set_low();
            silvia.until_unless("wait", BACKFLUSH_PAUSE_MILLIS, StopReason::Cancel)?;
        }
        Ok(())
    }
}
