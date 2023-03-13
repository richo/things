use crate::{Brew, Silvia, Operation};

mod straight;
pub use straight::StraightBrew;

mod richo;
pub use richo::RichoBrew;

mod preinfuse;
pub use preinfuse::PreInfuse;

mod backflush;
pub use backflush::BackFlush;

mod valveopen;
pub use valveopen::ValveOpen;

mod repro;
pub use repro::Repro;

#[derive(Clone, Copy)]
pub enum BrewContainer {
    Richo,
    PreInfuse,
    Straight,
    BackFlush,
    ValveOpen,
    Repro,
}

impl Default for BrewContainer {
    fn default() -> Self {
        Self::Richo
    }
}

impl BrewContainer {
    pub fn next(&self) -> BrewContainer {
        match self {
            BrewContainer::Richo => {
                BrewContainer::PreInfuse
            },
            BrewContainer::PreInfuse => {
                BrewContainer::Straight
            },
            BrewContainer::Straight => {
                BrewContainer::BackFlush
            },
            BrewContainer::BackFlush => {
                BrewContainer::ValveOpen
            },
            BrewContainer::ValveOpen => {
                BrewContainer::Repro
            },
            BrewContainer::Repro => {
                BrewContainer::Richo
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BrewContainer::Richo => {
                RichoBrew::NAME
            },
            BrewContainer::PreInfuse => {
                PreInfuse::NAME
            },
            BrewContainer::Straight => {
                StraightBrew::NAME
            },
            BrewContainer::BackFlush => {
                BackFlush::NAME
            },
            BrewContainer::ValveOpen => {
                ValveOpen::NAME
            },
            BrewContainer::Repro => {
                Repro::NAME
            }
        }
    }

    pub fn brew(&self, silvia: &mut Silvia) -> Result<Operation, Operation> {
        match self {
            BrewContainer::Richo => {
                RichoBrew::brew(silvia)
            },
            BrewContainer::PreInfuse => {
                PreInfuse::brew(silvia)
            },
            BrewContainer::Straight => {
                StraightBrew::brew(silvia)
            },
            BrewContainer::BackFlush => {
                BackFlush::brew(silvia)
            },
            BrewContainer::ValveOpen => {
                ValveOpen::brew(silvia)
            },
            BrewContainer::Repro => {
                Repro::brew(silvia)
            },
        }
    }
}

