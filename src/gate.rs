//! Defines if an `Event` should be triggered or not for a given `Pulse`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

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

    /// Create a new cloned instance of this gate. This actualy is a clone(), wrapped into
    /// a `Rc<RefCell<dyn Gate>>`, but called 'duplicate' to avoid conflicts with possible
    /// Clone impls.
    fn duplicate(&self) -> Rc<RefCell<dyn Gate>>;

    /// Resets the gate's internal state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// Probability gate implementation. Returns false for 0 pulse values and true for values of 1.
/// Values inbetween 0 and 1 do *maybe* trigger, using the pulse value as probability.
///  
/// The given rand::Rgn template class parameter / instance is used for generating random numbers
/// for probability checks.
#[derive(Debug)]
pub struct ProbabilityGate<R: Rng + Debug + Clone + 'static> {
    rand_gen: R,
    initial_rand_gen: Option<R>,
}

impl<R: Rng + Debug + Clone> ProbabilityGate<R> {
    pub fn new(rand_gen: R, is_seeded: bool) -> Self {
        // memorize the random number generator for reset(), but onlt when it's seeded
        let initial_rand_gen = if is_seeded {
            Some(rand_gen.clone())
        } else {
            None
        };
        Self {
            rand_gen,
            initial_rand_gen,
        }
    }
}

impl<R: Rng + Debug + Clone> Clone for ProbabilityGate<R> {
    fn clone(&self) -> Self {
        let mut clone = Self {
            rand_gen: self.rand_gen.clone(),
            initial_rand_gen: self.initial_rand_gen.clone(),
        };
        if clone.initial_rand_gen.is_none() {
            // when our random number generator is not seeded, 
            // enusure the clone uses a new rand state
            clone.rand_gen.next_u64();
        }
        clone
    }
}

impl<R: Rng + Debug + Clone> Gate for ProbabilityGate<R> {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn run(&mut self, pulse: &PulseIterItem) -> bool {
        pulse.value >= 1.0 || (pulse.value > 0.0 && pulse.value > self.rand_gen.gen_range(0.0..1.0))
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Gate>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset random number generator to its initial state when the gate is seeded
        if let Some(ref mut initial_rand_get) = self.initial_rand_gen {
            self.rand_gen = initial_rand_get.clone();
        }
    }
}
