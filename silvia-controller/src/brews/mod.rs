use crate::{Brew, Silvia, Operation};

mod straight;
pub use straight::StraightBrew;
mod richo;
pub use richo::RichoBrew;
mod preinfuse;
pub use preinfuse::PreInfuse;

mod backflush;
pub use backflush::BackFlush;

#[derive(Clone, Copy)]
pub enum BrewContainer {
    Richo,
    PreInfuse,
    Straight,
    BackFlush,
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
                BrewContainer::Richo
            },
        }
    }

    pub fn get(&self) -> &dyn Brew {
        match self {
            BrewContainer::Richo => {
                &RichoBrew
            },
            BrewContainer::PreInfuse => {
                &PreInfuse
            },
            BrewContainer::Straight => {
                &StraightBrew
            },
            BrewContainer::BackFlush => {
                &BackFlush
            },
        }
    }
}

