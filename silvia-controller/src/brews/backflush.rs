use crate::{Brew, Silvia, until_unless, Conclusion};

/// Backflush the machine
pub struct BackFlush;

/// Backflush the machine.
const BACKFLUSH_REPEATS: u16 = 5;
const BACKFLUSH_ON_MILLIS: u16 = 5000;
const BACKFLUSH_PAUSE_MILLIS: u16 = 7000;

impl Brew for BackFlush {
    const LOGLINE: &'static str = "backflush";
    fn brew(silvia: &mut Silvia) -> Conclusion {
        for _ in 0..BACKFLUSH_REPEATS {
            silvia.valve.set_high();
            silvia.pump.set_high();
            let flush = |time| { let _ = ufmt::uwriteln!(silvia.serial, "flush {}",  time); };
            let res = until_unless(BACKFLUSH_ON_MILLIS, || silvia.backflush.is_low(), flush);
            silvia.pump.set_low();
            silvia.valve.set_low();
            let wait = |time| { let _ = ufmt::uwriteln!(silvia.serial, "wait {}",  time); };
            if let Conclusion::Finished = res {
                until_unless(BACKFLUSH_PAUSE_MILLIS, || silvia.backflush.is_low(), wait);
            } else {
                return res
            }
        }
        Conclusion::Finished
    }
}
