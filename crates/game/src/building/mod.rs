//! Block placement and breaking systems.

mod airlock;
mod base;
mod placement;

pub use airlock::{Airlock, AirlockState, PowerGenerator, PowerSystem};
pub use base::{BaseCompartment, PressureWindow};
pub use placement::BlockInteraction;
