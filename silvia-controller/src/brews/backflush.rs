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
            let flush = |time| { let _ = ufmt::uwriteln!(silvia.serial, "flush {}",  time); };
            silvia.valve.set_high();
            silvia.pump.set_high();
            until_unless(BACKFLUSH_ON_MILLIS, || silvia.backflush.is_low(), flush)?;

            let wait = |time| { let _ = ufmt::uwriteln!(silvia.serial, "wait {}",  time); };
            silvia.pump.set_low();
            silvia.valve.set_low();
            until_unless(BACKFLUSH_PAUSE_MILLIS, || silvia.backflush.is_low(), wait)?;
        }
        Ok(())
    }
}
