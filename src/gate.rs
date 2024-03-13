//! Defines if an `Event` should be triggered or not for a given `Pulse`.

use std::fmt::Debug;

use crate::{BeatTimeBase, PulseIterItem};
use rand::Rng;

// -------------------------------------------------------------------------------------------------

/// Defines if an [Event](crate::Event) should be triggered or not, depending on an incoming
/// [Pulse](PulseIterItem) value.
pub trait Gate: Debug {
    /// Set or update the gate's internal beat or second time base with the new time base.
    /// Note: SampleTimeBase can be derived from BeatTimeBase via `SecondTimeBase::from(beat_time)`
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Returns true if the event should be triggered, else false.
    fn run(&mut self, pulse: &PulseIterItem) -> bool;
}

// -------------------------------------------------------------------------------------------------

/// Probability gate implementation. Returns false for 0 pulse values and true for values of 1. 
/// Values inbetween 0 and 1 do *maybe* trigger, using the pulse value as probability.
///  
/// The given rand::Rgn template class parameter / instance is used for generating random numbers
/// for probability checks.
#[derive(Debug, Clone)]
pub struct ProbabilityGate<R: Rng + Debug + Clone> {
    rand_gen: R,
}

impl<R: Rng + Debug + Clone> ProbabilityGate<R> {
    pub fn new(rand_gen: R) -> Self {
        Self { rand_gen }
    }
}

impl<R: Rng + Debug + Clone> Gate for ProbabilityGate<R> {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn run(&mut self, pulse: &PulseIterItem) -> bool {
        pulse.value >= 1.0 || (pulse.value > 0.0 && pulse.value > self.rand_gen.gen_range(0.0..1.0))
    }
}
