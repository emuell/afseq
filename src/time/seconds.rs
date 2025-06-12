use crate::{
    time::{SampleTimeBase, SampleTimeDisplay},
    SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Second time base for wallclock time based [Pattern](`crate::Pattern`) impls.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct SecondTimeBase {
    pub samples_per_sec: u32,
}

impl SampleTimeBase for SecondTimeBase {
    #[inline]
    fn samples_per_second(&self) -> u32 {
        self.samples_per_sec
    }
}

impl SampleTimeDisplay for SecondTimeBase {
    /// Generate a second string representation of the the given sample time.
    fn display(&self, sample_time: SampleTime) -> String {
        format!("{}s", sample_time as f32 / self.samples_per_second() as f32)
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines a number of seconds in [`SecondTimeBase`].
pub type SecondTimeStep = f64;
