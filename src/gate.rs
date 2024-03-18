//! Defines if an `Event` should be triggered or not for a given `Pulse`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::{BeatTimeBase, PulseIterItem};

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
#[derive(Debug, Clone)]
pub struct ProbabilityGate {
    rand_gen: Xoshiro256PlusPlus,
    seed: Option<[u8; 32]>,
}

impl ProbabilityGate {
    pub fn new(seed: Option<[u8; 32]>) -> Self {
        let rand_seed = seed.unwrap_or_else(|| thread_rng().gen());
        let rand_gen = Xoshiro256PlusPlus::from_seed(rand_seed);
        Self { rand_gen, seed }
    }
}

impl Gate for ProbabilityGate {
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
        if let Some(seed) = self.seed {
            self.rand_gen = Xoshiro256PlusPlus::from_seed(seed);
        }
        // else create a new random number generator from a random seed
        else {
            self.rand_gen = Xoshiro256PlusPlus::from_seed(thread_rng().gen());
        }
    }
}
