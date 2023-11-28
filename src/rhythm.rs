//! Periodically emit `Events` via an `EventIter` with a given time base.

use std::fmt::Debug;

use crate::{event::Event, time::SampleTimeDisplay, BeatTimeBase, SampleTime};

pub mod beat_time;
pub mod euclidean;
pub mod second_time;

// -------------------------------------------------------------------------------------------------

/// A `Rhythm` is an iterator which emits optional [`Event`] in sample-rate resolution.
///
/// It triggers events periodically, producing events at a specific sample time.
/// An audio player can then use the sample time to schedule those events within the audio stream.
///
/// Rhythm impls typically will use a [EventIter][`super::EventIter`] to produce one or multiple
/// note or parameter change events. The event iter impl is an iterator too, so the emitted content
/// may dynamically change over time as well.
pub trait Rhythm: Iterator<Item = (SampleTime, Option<Event>)> + Debug {
    /// Create a time display printer, which serializes the given sample time to the Rhythm's
    /// time base as appropriated (in seconds or beats).
    fn time_display(&self) -> Box<dyn SampleTimeDisplay>;
    /// Update the rhythms internal beat or second time bases with the new time base.
    /// Note: SampleTimeBase can be derived from BeatTimeBase via `Into::<SampleTimeBase>(beat_time)`
    fn set_time_base(&mut self, time_base: BeatTimeBase);

    /// Length in samples of a single step in the rhythm's pattern.
    fn samples_per_step(&self) -> f64;
    /// Get length of the rhythm's internal pattern (cycle length in steps).
    /// A rhythm pattern repeats after self.samples_per_step() * self.pattern_length() samples.
    fn pattern_length(&self) -> usize;

    /// Custom sample offset value which is applied to emitted events.
    fn sample_offset(&self) -> SampleTime;
    /// Set a new custom sample offset value.
    fn set_sample_offset(&mut self, sample_offset: SampleTime);

    /// Resets/rewinds the rhythm to its initial state.
    fn reset(&mut self);
}
