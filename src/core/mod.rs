//! Core mathematical and profile structures for OpenVector.

pub mod accumulator;
pub mod math;
pub mod profile;

pub use accumulator::SubpixelAccumulator;
pub use math::{FourierModel, PolarVector, Angle};
pub use profile::{Profile, ProfileManager, RawAccelConfig};
