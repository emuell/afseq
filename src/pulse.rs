//! Pulse within a pattern as used by `Pattern`.

// -------------------------------------------------------------------------------------------------

pub type PulseValue = f32;
pub type PulseStepTime = f64;

// -------------------------------------------------------------------------------------------------

/// Representes single pulse event or a sub division of pulses in patterns.
///
/// When the pattern is played, each pulse uses the duration of a single step as it's base duration,
/// so a single pulse event will last exactly one step, but a SubDivision pulse vector will convert
/// the entire steps duration too, so each pulse in the sub division gets fold into the parent step's
/// timing.
///  
/// Using SubDivision thus can use fractions of a rhythms base time step.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pulse {
    Pulse(PulseValue),
    SubDivisions(Vec<Pulse>),
}

impl Pulse {
    /// Returns true when the pulse is a subdivision and the subdivision is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of pulses in the underlying pulse.
    pub fn len(&self) -> usize {
        match self {
            Pulse::Pulse(_) => 1,
            Pulse::SubDivisions(sub) => sub.len(),
        }
    }
}

/// Converts a boolean to a pulse.
impl From<bool> for Pulse {
    fn from(val: bool) -> Self {
        Pulse::Pulse(val as u8 as PulseValue)
    }
}

/// Converts an integer to a pulse.
impl From<u32> for Pulse {
    fn from(value: u32) -> Self {
        Pulse::Pulse(value as PulseValue)
    }
}

/// Converts a float to a pulse.
impl From<f32> for Pulse {
    fn from(value: f32) -> Self {
        Pulse::Pulse(value as PulseValue)
    }
}

/// Converts a vector of booleans to a pulse sub division.
impl From<Vec<bool>> for Pulse {
    fn from(values: Vec<bool>) -> Self {
        Pulse::SubDivisions(values.into_iter().map(|p| p.into()).collect())
    }
}

/// Converts a vector of integers to a pulse sub division.
impl From<Vec<u32>> for Pulse {
    fn from(values: Vec<u32>) -> Self {
        Pulse::SubDivisions(values.into_iter().map(|p| p.into()).collect())
    }
}

/// Converts a vector of floats to a pulse sub division.
impl From<Vec<f32>> for Pulse {
    fn from(values: Vec<f32>) -> Self {
        Pulse::SubDivisions(values.into_iter().map(Pulse::Pulse).collect())
    }
}

/// Converts a vector of pulses to a pulse sub division.
impl From<Vec<Pulse>> for Pulse {
    fn from(values: Vec<Pulse>) -> Self {
        Pulse::SubDivisions(values)
    }
}

// -------------------------------------------------------------------------------------------------

/// An iterator over a pulse, recursively flattening all subdivisions.
#[derive(Clone, Debug)]
pub struct PulseIter {
    flattened: Vec<(PulseValue, PulseStepTime)>,
    index: usize,
}

impl PulseIter {
    pub fn new(pulse: Pulse) -> Self {
        let mut flattened = vec![];
        Self::flatten(&mut flattened, &pulse, 1.0);
        Self {
            flattened,
            index: 0,
        }
    }

    fn flatten(
        result: &mut Vec<(PulseValue, PulseStepTime)>,
        current_pulse: &Pulse,
        step_time: PulseStepTime,
    ) {
        match current_pulse {
            Pulse::Pulse(value) => result.push((*value, step_time)),
            Pulse::SubDivisions(ref sub_pulses) => {
                for sub_pulse in sub_pulses {
                    let sub_step_time = step_time / sub_pulses.len() as f64;
                    Self::flatten(result, sub_pulse, sub_step_time);
                }
            }
        }
    }
}

impl Iterator for PulseIter {
    type Item = (PulseValue, PulseStepTime);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.flattened.len() {
            None
        } else {
            let item = self.flattened[self.index];
            self.index += 1;
            Some(item)
        }
    }
}

/// Converts a pulse to an iterator.
impl IntoIterator for Pulse {
    type Item = (PulseValue, PulseStepTime);
    type IntoIter = PulseIter;

    fn into_iter(self) -> Self::IntoIter {
        PulseIter::new(self)
    }
}
