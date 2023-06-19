use crate::{Brew, Silvia, Conclusion, StopReason, Count, OperationExt, discard};

/// Backflush the machine
pub struct BackFlush;

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 7000;

impl Brew for BackFlush {
    const NAME: &'static str = "backflush";

    fn brew(silvia: &mut Silvia) -> Conclusion {
        let mut extra = [b' ', b'0', b'/', b'0' + BACKFLUSH_REPEATS as u8];
        for i in 0..BACKFLUSH_REPEATS {
            extra[1] += 1;
            discard(silvia.write_extra(&extra));
            silvia.valve_on();
            silvia.pump_on();
            silvia.until_unless("flush", BACKFLUSH_ON_MILLIS, StopReason::Cancel, Count::DownFrom(BACKFLUSH_ON_MILLIS as u32))?;

            if i != BACKFLUSH_REPEATS - 1 {
                silvia.pump_off();
                silvia.valve_off();
                silvia.until_unless("wait", BACKFLUSH_PAUSE_MILLIS, StopReason::Cancel, Count::DownFrom(BACKFLUSH_PAUSE_MILLIS as u32))?;
            }
        }
        Conclusion::done()
    }
}
