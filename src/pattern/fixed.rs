use std::{cell::RefCell, rc::Rc};

use crate::{BeatTimeBase, Pattern, Pulse, PulseIter, PulseStepTime, PulseValue};

// -------------------------------------------------------------------------------------------------

/// A pattern which endlessly emits pulses by stepping through a fixed pulse array.
#[derive(Clone, Debug)]
pub struct FixedPattern {
    pulses: Vec<Pulse>,
    pulse_iter: Option<PulseIter>,
    step: usize,
}

impl Default for FixedPattern {
    fn default() -> Self {
        Self::from_pulses(vec![Pulse::Pulse(1.0)])
    }
}

impl FixedPattern {
    /// Create a pattern from a vector of pattern pulses or a vector of values which can be
    /// converted to pattern pulses (boolean, u32, f32).
    pub fn from_pulses<T>(pulses: Vec<T>) -> Self
    where
        Pulse: std::convert::From<T> + Sized,
    {
        let pulses = pulses
            .into_iter()
            .map(|v| Pulse::from(v))
            .collect::<Vec<_>>();
        let pulse_iter = pulses.first().map(|pulse| pulse.clone().into_iter());
        let step = 0;
        FixedPattern {
            pulses,
            pulse_iter,
            step,
        }
    }
}

impl Pattern for FixedPattern {
    fn len(&self) -> usize {
        self.pulses.len()
    }

    fn run(&mut self) -> (PulseValue, PulseStepTime) {
        assert!(!self.is_empty(), "Can't run empty patterns");
        // if we have a pulse iterator, consume it
        if let Some(pulse_iter) = &mut self.pulse_iter {
            if let Some(pulse) = pulse_iter.next() {
                return pulse;
            }
        }
        // else move on to the next pulse
        self.step += 1;
        if self.step >= self.pulses.len() {
            self.step = 0;
        }
        self.pulse_iter = Some(self.pulses[self.step].clone().into_iter());
        self.pulse_iter
            .as_mut()
            .unwrap()
            .next()
            .unwrap_or((0.0, 1.0))
    }

    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Pattern>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        self.step = 0;
        self.pulse_iter = self.pulses.first().map(|pulse| pulse.clone().into_iter());
    }
}

// -------------------------------------------------------------------------------------------------

/// Create `FixedPattern` from convertible types.
pub trait ToFixedPattern {
    fn to_pattern(self) -> FixedPattern;
}

impl<T> ToFixedPattern for Vec<T>
where
    Pulse: std::convert::From<T>,
{
    /// Create a vector of pulses, numbers or booleans to a new [`FixedPattern`].
    fn to_pattern(self) -> FixedPattern {
        FixedPattern::from_pulses(self)
    }
}

impl<const N: usize, T> ToFixedPattern for [T; N]
where
    Pulse: std::convert::From<T>,
{
    /// Create a static array of pulses, numbers or booleans to a new [`FixedPattern`].
    fn to_pattern(self) -> FixedPattern {
        FixedPattern::from_pulses(self.into_iter().collect::<Vec<_>>())
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn run() {
        let mut pattern = [1.0, 0.0, 1.0, 0.0].to_pattern();
        assert_eq!(
            vec![pattern.run(), pattern.run(), pattern.run(), pattern.run()],
            vec![(1.0, 1.0), (0.0, 1.0), (1.0, 1.0), (0.0, 1.0)]
        );

        pattern = [
            Pulse::from(1.0),
            Pulse::from(0.0),
            Pulse::from(vec![1.0, 1.0]),
            Pulse::from(0.0),
        ]
        .to_pattern();
        assert_eq!(
            vec![
                pattern.run(),
                pattern.run(),
                pattern.run(),
                pattern.run(),
                pattern.run()
            ],
            vec![(1.0, 1.0), (0.0, 1.0), (1.0, 0.5), (1.0, 0.5), (0.0, 1.0)]
        );

        pattern = [
            Pulse::from(1.0),
            Pulse::from(0.0),
            Pulse::from(vec![Pulse::from(vec![0.0, 1.0]), Pulse::from(1.0)]),
            Pulse::from(0.0),
        ]
        .to_pattern();
        assert_eq!(
            vec![
                pattern.run(),
                pattern.run(),
                pattern.run(),
                pattern.run(),
                pattern.run(),
                pattern.run()
            ],
            vec![
                (1.0, 1.0),
                (0.0, 1.0),
                (0.0, 0.25),
                (1.0, 0.25),
                (1.0, 0.5),
                (0.0, 1.0)
            ]
        );
    }
}
