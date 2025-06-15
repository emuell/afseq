//! Pulse event within a `Pattern`.

// -------------------------------------------------------------------------------------------------

/// Represents a single pulse value or a sub division of pulse values in a [`Rhythm`](crate::Rhythm).
///
/// When a rhythm plays, each single pulse or the entire pulse subdivision uses the duration of
/// a single time step as base duration, so a single pulse event will last exactly one step,
/// and all pulses in a sub division cover the entire step's duration as well.
///
/// By using pulses with sub divisions, complex sub rhythms can be created without increasing
/// the pattern's base time resolution.
///
/// ### Example
///
/// ```rust
/// use pattrns::Pulse;
/// // Assuming pulse step is 1 beat.
/// // Defines a pulse rhythm with one quater note followed by a 16th note triplet.
/// let pulses = vec![Pulse::from(1), Pulse::from(vec![1, 1, 1])];
/// ````
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Pulse {
    Pulse(f32),
    SubDivision(Vec<Pulse>),
}

impl Pulse {
    /// Returns true when the pulse is a sub division and empty, else false.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of pulses in the underlying sub division or 1 for single pulses.
    pub fn len(&self) -> usize {
        match self {
            Pulse::Pulse(_) => 1,
            Pulse::SubDivision(sub_div) => sub_div.iter().fold(0, |sum, pulse| sum + pulse.len()),
        }
    }
}

/// Converts a boolean value to a pulse.
impl From<bool> for Pulse {
    fn from(value: bool) -> Self {
        Pulse::Pulse(value as u8 as f32)
    }
}

/// Converts an integer value to a pulse.
impl From<u32> for Pulse {
    fn from(value: u32) -> Self {
        Pulse::Pulse(value as f32)
    }
}

/// Converts a float value to a pulse.
impl From<f32> for Pulse {
    fn from(value: f32) -> Self {
        Pulse::Pulse(value)
    }
}

/// Converts a vector of boolean values to a pulse sub division.
impl From<Vec<bool>> for Pulse {
    fn from(values: Vec<bool>) -> Self {
        Pulse::SubDivision(values.into_iter().map(Pulse::from).collect())
    }
}

/// Converts a vector of integer values to a pulse sub division.
impl From<Vec<u32>> for Pulse {
    fn from(values: Vec<u32>) -> Self {
        Pulse::SubDivision(values.into_iter().map(Pulse::from).collect())
    }
}

/// Converts a vector of float value to a pulse sub division.
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
