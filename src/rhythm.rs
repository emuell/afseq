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

#[cfg(feature = "tidal")]
pub mod tidal;

// -------------------------------------------------------------------------------------------------

/// A `RhythmSampleIter` is an iterator which emits optional [`Event`] in sample-rate resolution.
///
/// It triggers events periodically, producing events at a specific sample time.
/// An audio player can then use the sample time to schedule those events within the audio stream.
///
/// RhythmIterator impls typically will use a [EventIter][`super::EventIter`] to produce one or
/// multiple note or parameter change events. The event iter impl is an iterator too, so the
/// emitted content may dynamically change over time as well.
pub trait RhythmSampleIter: Iterator<Item = (SampleTime, Option<Event>)> + Debug {
    /// Create a sample time display printer, which serializes the given sample time to the Rhythm's
    /// time base as appropriated (in seconds or beats). May be useful for debugging purposes.
    fn sample_time_display(&self) -> Box<dyn SampleTimeDisplay>;

    /// Custom sample offset value which is applied to all emitted events.
    fn sample_offset(&self) -> SampleTime;
    /// Set a new custom sample offset value.
    fn set_sample_offset(&mut self, sample_offset: SampleTime);

    /// Sample time iter: returns self.next() up and until the given sample time
    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)>;
}

// -------------------------------------------------------------------------------------------------

/// A `Rhythm` is a dyn clonable `RhythmSampleIter` with instrument and time base access.
///
/// Rhythms can be reset and cloned (duplicated), so that they can be triggered multiple times using
/// possibly different patterns and time bases.
pub trait Rhythm: RhythmSampleIter {
    /// Length in samples of a single step in the rhythm's internal pattern.
    fn pattern_step_length(&self) -> f64;
    /// Get length in steps of the rhythm's internal pattern (cycle length in steps).
    /// A rhythm pattern repeats after self.pattern_step_length() * self.pattern_length() samples.
    fn pattern_length(&self) -> usize;

    /// Set or update the rhythm's internal beat or second time bases with a new time base.
    /// Note: SampleTimeBase can be derived from BeatTimeBase via `SecondTimeBase::from(beat_time)`
    fn set_time_base(&mut self, time_base: &BeatTimeBase);
    /// Set/unset a new default instrument value for all emitted note events which have no
    /// instrument value set.
    fn set_instrument(&mut self, instrument: Option<InstrumentId>);

    /// Create a new cloned instance of this rhythm. This actualy is a clone(), wrapped into
    /// a `Rc<RefCell<dyn Rhythm>>`, but called 'duplicate' to avoid conflicts with possible Clone impls.
    fn duplicate(&self) -> Rc<RefCell<dyn Rhythm>>;
    /// Resets/rewinds the rhythm to its initial state.
    fn reset(&mut self);
}
