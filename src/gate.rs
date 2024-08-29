//! Defines if an `Event` should be triggered or not for a given `Pulse`.

use std::{borrow::Cow, fmt::Debug};

use crate::{BeatTimeBase, InputParameterMap, PulseIterItem};

// -------------------------------------------------------------------------------------------------

pub mod probability;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Defines if an [Event](crate::Event) should be triggered or not, depending on an incoming
/// [Pulse](PulseIterItem) value.
pub trait Gate: Debug {
    /// Set or update the gate's internal beat or second time base with the new time base.
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Set or update optional, application specific external context data for the gate.
    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]);

    /// Set or update optional, input parameter map for callbacks.
    fn set_input_parameters(&mut self, parameters: InputParameterMap);

    /// Returns true if the event should be triggered, else false.
    fn run(&mut self, pulse: &PulseIterItem) -> bool;

    /// Create a new cloned instance of this gate. This actualy is a clone(), wrapped into
    /// a `Box<dyn Gate>`, but called 'duplicate' to avoid conflicts with possible
    /// Clone impls.
    fn duplicate(&self) -> Box<dyn Gate>;

    /// Resets the gate's internal state.
    fn reset(&mut self);
}
