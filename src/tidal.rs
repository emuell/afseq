//! Tidal mini parser and event generator, used as `EventIter`.

mod cycle;
pub use cycle::{Cycle, Event, Pitch, Span, Target, Value};
