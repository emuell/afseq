use rand::{rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::{BeatTimeBase, Event, Gate, ParameterSet, RhythmEvent};

// -------------------------------------------------------------------------------------------------

/// Probability gate implementation. Returns false for 0 pulse values and true for values of 1.
/// Values between 0 and 1 do *maybe* trigger, using the pulse value as probability.
#[derive(Debug, Clone)]
pub struct ProbabilityGate {
    rand_gen: Xoshiro256PlusPlus,
    seed: Option<u64>,
}

impl ProbabilityGate {
    pub fn new(seed: Option<u64>) -> Self {
        let rand_seed = seed.unwrap_or_else(|| rng().random());
        let rand_gen = Xoshiro256PlusPlus::seed_from_u64(rand_seed);
        Self { rand_gen, seed }
    }
}

impl Gate for ProbabilityGate {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_parameters(&mut self, _parameters: ParameterSet) {
        // nothing to do
    }

    fn run(&mut self, pulse: &RhythmEvent) -> bool {
        pulse.value >= 1.0
            || (pulse.value > 0.0 && pulse.value > self.rand_gen.random_range(0.0..1.0))
    }

    fn duplicate(&self) -> Box<dyn Gate> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // reset random number generator to its initial state, when the gate is seeded
        if let Some(seed) = self.seed {
            self.rand_gen = Xoshiro256PlusPlus::seed_from_u64(seed);
        }
    }
}
