use crate::{Brew, Silvia, until_unless, Conclusion};

/// richo's playground brew
pub struct RichoBrew;

impl Brew for RichoBrew {
    const LOGLINE: &'static str = "straight";
    fn brew(silvia: &mut Silvia) -> Conclusion {
        silvia.valve.set_high();
        // Pulse pump on and off for 300/200 3 times
        for t in [200, 300, 400] {
            silvia.pump.set_high();
            let infuse = |time| { let _ = ufmt::uwriteln!(silvia.serial, "infuse {}",  time); };
            if let Conclusion::Interrupted(i) = until_unless(t, || silvia.brew.is_low(), infuse) {
                silvia.pump.set_low();
                return Conclusion::Interrupted(i);
            }
            silvia.pump.set_low();
            let wait = |time| { let _ = ufmt::uwriteln!(silvia.serial, "inwait {}",  time); };
            if let Conclusion::Interrupted(i) = until_unless(200, || silvia.brew.is_low(), wait) {
                silvia.pump.set_low();
                return Conclusion::Interrupted(i);
            }
        }

        // Run the main brew
        // Infuse leaves the valve closed, but we'll double check
        silvia.valve.set_high();
        silvia.pump.set_high();

        // We'll run the pump for 35s or until someone stops us
        let brew = |time| { let _ = ufmt::uwriteln!(silvia.serial, "brew {}",  time); };
        until_unless(35000, || silvia.brew.is_low(), brew)
    }
}
