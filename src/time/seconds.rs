use crate::{
    time::{SampleTimeDisplay, TimeBase},
    SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Second time timing base for beat based [Rhythm](`crate::Rhythm`) impls.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct SecondTimeBase {
    pub samples_per_sec: u32,
}

impl TimeBase for SecondTimeBase {
    #[inline]
    fn samples_per_second(&self) -> u32 {
        self.samples_per_sec
    }
}

impl SampleTimeDisplay for SecondTimeBase {
    /// generate a second string representation of the the given sample time
    fn display(&self, sample_time: SampleTime) -> String {
        format!("{}s", sample_time as f32 / self.samples_per_second() as f32)
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of seconds in [`SecondTimeBase`].
pub type SecondTimeStep = f64;
