use crate::{pattern::Pattern, BeatTimeBase};

// -------------------------------------------------------------------------------------------------

/// A pattern which endlessly emits pulses by stepping through a fixed pulse array.
#[derive(Clone, Debug)]
pub struct FixedPattern {
    pulses: Vec<f32>,
    step: usize,
}

impl Default for FixedPattern {
    fn default() -> Self {
        Self {
            pulses: vec![1.0],
            step: 0,
        }
    }
}

impl FixedPattern {
    /// Create a pattern from a vector. Param `pulses` is evaluated as an array of numbers:
    /// when to trigger an event and when not (0, 1), but can also be specified as boolean
    /// or integer array.
    pub fn from_vector<T>(pulses: Vec<T>) -> Self
    where
        f64: std::convert::TryFrom<T>,
    {
        let pulses = pulses
            .into_iter()
            .map(|f| f64::try_from(f).unwrap_or(0.0) as f32)
            .collect::<Vec<_>>();
        let step = 0;
        FixedPattern { pulses, step }
    }

    /// Create a pattern from a static array of numbers or booleans.
    pub fn from_array<const N: usize, T>(pulses: [T; N]) -> Self
    where
        f64: std::convert::TryFrom<T>,
    {
        let pulses = pulses
            .into_iter()
            .map(|f| f64::try_from(f).unwrap_or(0.0) as f32)
            .collect::<Vec<_>>();
        Self::from_vector::<f32>(pulses)
    }
}

impl Pattern for FixedPattern {
    fn len(&self) -> usize {
        self.pulses.len()
    }

    fn update_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn run(&mut self) -> f32 {
        assert!(!self.is_empty(), "Can't run empty patterns");
        let pulse = self.pulses[self.step];
        self.step += 1;
        if self.step >= self.pulses.len() {
            self.step = 0;
        }
        pulse
    }

    fn reset(&mut self) {
        self.step = 0;
    }
}

// -------------------------------------------------------------------------------------------------

/// Create `FixedPattern` from convertible types.
pub trait ToFixedPattern {
    fn to_pattern(self) -> FixedPattern;
}

impl<T> ToFixedPattern for Vec<T>
where
    f64: std::convert::TryFrom<T>,
{
    /// Wrap a vector of numbers or booleans to a new [`FixedPattern`].
    fn to_pattern(self) -> FixedPattern {
        FixedPattern::from_vector(self)
    }
}

impl<const N: usize, T> ToFixedPattern for [T; N]
where
    f64: std::convert::TryFrom<T>,
{
    /// Wrap a static array of numbers or booleans to a new [`FixedPattern`].
    fn to_pattern(self) -> FixedPattern {
        FixedPattern::from_array(self)
    }
}
