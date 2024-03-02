//! Periodically emit `Events` via an `EventIter` with a given time base on a
//! rhythmical pattern defined via a `Pattern`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    event::{Event, InstrumentId},
    time::SampleTimeDisplay,
    BeatTimeBase, SampleTime,
};

pub mod beat_time;
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
    /// Update the rhythm's internal beat or second time bases with the new time base.
    /// Note: SampleTimeBase can be derived from BeatTimeBase via `SecondTimeBase::from(beat_time)`
    fn update_time_base(&mut self, time_base: &BeatTimeBase);

    /// Get default instrument value for note events.
    fn instrument(&self) -> Option<InstrumentId>;
    /// Set/unset a new default instrument value for all emitted note events which have no
    /// instrument value set.
    fn set_instrument(&mut self, instrument: Option<InstrumentId>);

    /// Length in samples of a single step in the rhythm's pattern.
    fn samples_per_step(&self) -> f64;
    /// Get length of the rhythm's internal pattern (cycle length in steps).
    /// A rhythm pattern repeats after self.samples_per_step() * self.pattern_length() samples.
    fn pattern_length(&self) -> usize;

    /// Custom sample offset value which is applied to all emitted events.
    fn sample_offset(&self) -> SampleTime;
    /// Set a new custom sample offset value.
    fn set_sample_offset(&mut self, sample_offset: SampleTime);
    /// Sample time iter: returns self.next() up and until the given sample time
    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)>;

    /// Create a new instance of this rhythm.
    fn clone_dyn(&self) -> Rc<RefCell<dyn Rhythm>>;
    /// Resets/rewinds the rhythm to its initial state.
    fn reset(&mut self);
}
