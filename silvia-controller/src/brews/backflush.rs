use crate::{Brew, Silvia, Conclusion, StopReason, Count, OperationExt};

/// Backflush the machine
pub struct BackFlush;

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 7000;

impl Brew for BackFlush {
    const NAME: &'static str = "backflush";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        for _ in 0..BACKFLUSH_REPEATS {
            silvia.valve_on();
            silvia.pump_on();
            silvia.until_unless("flush", BACKFLUSH_ON_MILLIS, StopReason::Cancel, Count::DownFrom(BACKFLUSH_ON_MILLIS as u32))?;

            silvia.pump_off();
            silvia.valve_off();
            silvia.until_unless("wait", BACKFLUSH_PAUSE_MILLIS, StopReason::Cancel, Count::DownFrom(BACKFLUSH_PAUSE_MILLIS as u32))?;
        }
        Conclusion::finished(666)
    }
}
