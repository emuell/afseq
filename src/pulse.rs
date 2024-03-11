//! Pulse event within a `Pattern`.

// -------------------------------------------------------------------------------------------------

/// Represents a single pulse event or a sub division of pulse events in a pattern step.
///
/// When a pattern is played, each pulse or the subdivision use the duration of a single step as
/// its base duration, so a single pulse event will last exactly one step, but pulses in a sub
/// division pulse vector will cover the entire step's duration too.
///
/// By using pulses with sub divisions, one can create any kind of complex rhythms without
/// increasing the step resolution.
///
/// # Examples:
///
/// ```rust
/// use afseq::Pulse;
/// // Assuming pulse step is 1 beat.
/// // Defines a pattern with one quater note followed by a 16th note triplet.
/// let pattern = vec![Pulse::from(1), Pulse::from(vec![1, 1, 1])];
/// ````
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pulse {
    Pulse(f32),
    SubDivision(Vec<Pulse>),
}

impl Pulse {
    /// Returns true when the pulse is a sub division and the sub division is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of pulses in the underlying pulse.
    pub fn len(&self) -> usize {
        match self {
            Pulse::Pulse(_) => 1,
            Pulse::SubDivision(sub_div) => sub_div.iter().fold(0, |sum, pulse| sum + pulse.len()),
        }
    }

    /// Returns a flattened copy of all underlying pulse values.
    pub fn flattened(&self) -> Vec<PulseIterItem> {
        let mut values = vec![];
        self.expand_into(&mut values, 1.0);
        values
    }

    fn expand_into(&self, result: &mut Vec<PulseIterItem>, step_time: f64) {
        match self {
            Pulse::Pulse(value) => {
                let value = *value;
                result.push(PulseIterItem { value, step_time });
            }
            Pulse::SubDivision(ref sub_pulses) => {
                for sub_pulse in sub_pulses {
                    let sub_step_time = step_time / sub_pulses.len() as f64;
                    sub_pulse.expand_into(result, sub_step_time);
                }
            }
        }
    }
}

/// Converts a boolean to a pulse.
impl From<bool> for Pulse {
    fn from(value: bool) -> Self {
        Pulse::Pulse(value as u8 as f32)
    }
}

/// Converts an integer to a pulse.
impl From<u32> for Pulse {
    fn from(value: u32) -> Self {
        Pulse::Pulse(value as f32)
    }
}

/// Converts a float to a pulse.
impl From<f32> for Pulse {
    fn from(value: f32) -> Self {
        Pulse::Pulse(value)
    }
}

/// Converts a vector of booleans to a pulse sub division.
impl From<Vec<bool>> for Pulse {
    fn from(values: Vec<bool>) -> Self {
        Pulse::SubDivision(values.into_iter().map(Pulse::from).collect())
    }
}

/// Converts a vector of integers to a pulse sub division.
impl From<Vec<u32>> for Pulse {
    fn from(values: Vec<u32>) -> Self {
        Pulse::SubDivision(values.into_iter().map(Pulse::from).collect())
    }
}

/// Converts a vector of floats to a pulse sub division.
impl From<Vec<f32>> for Pulse {
    fn from(values: Vec<f32>) -> Self {
        Pulse::SubDivision(values.into_iter().map(Pulse::from).collect())
    }
}

/// Converts a vector of pulses to a pulse sub division.
impl From<Vec<Pulse>> for Pulse {
    fn from(values: Vec<Pulse>) -> Self {
        Pulse::SubDivision(values)
    }
}

// -------------------------------------------------------------------------------------------------

/// Iterator item of PulseIter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PulseIterItem {
    /// Pulse value in a pattern in range \[0 - 1\] where 0 means don't trigger anything, and 1
    /// means definitely do trigger something. Values within 0 - 1 maybe trigger, depending on the
    /// pattern/rhythm impl.
    pub value: f32,
    /// Pulse step time fraction in range \[0 - 1\]. 1 means advance by a full step, 0.5 means
    /// advance by a half step, etc.
    pub step_time: f64,
}

impl Default for PulseIterItem {
    fn default() -> Self {
        Self {
            value: 0.0,
            step_time: 1.0,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// An iterator over a pulse, recursively flattening all subdivisions.
#[derive(Clone, Debug)]
pub struct PulseIter {
    pulse_values: Vec<PulseIterItem>,
    pulse_step: usize,
}

impl PulseIter {
    pub fn new(pulse: Pulse) -> Self {
        let pulse_values = pulse.flattened();
        let pulse_step = 0;
        Self {
            pulse_values,
            pulse_step,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.pulse_values.len()
    }

    pub fn step(&self) -> usize {
        self.pulse_step
    }
}

impl Iterator for PulseIter {
    type Item = PulseIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pulse_step >= self.pulse_values.len() {
            None
        } else {
            let pulse = self.pulse_values[self.pulse_step];
            self.pulse_step += 1;
            Some(pulse)
        }
    }
}

impl IntoIterator for Pulse {
    type Item = PulseIterItem;
    type IntoIter = PulseIter;

    fn into_iter(self) -> Self::IntoIter {
        PulseIter::new(self)
    }
}
