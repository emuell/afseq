//! Periodically emit `Events` via an `EventIter` with a given time base on a
//! rhythmical pattern defined via a `Pattern`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    event::{Event, InstrumentId},
    time::SampleTimeDisplay,
    BeatTimeBase, SampleTime,
};

// -------------------------------------------------------------------------------------------------

pub mod beat_time;
mod generic;
pub mod second_time;

#[cfg(feature = "tidal")]
pub mod tidal;

#[cfg(doc)]
use super::EventIter;

// -------------------------------------------------------------------------------------------------

/// A `RhythmIter` is an iterator which emits optional [`Event`] in sample-rate resolution.
///
/// It triggers events periodically, producing events at at specific sample times.
/// An audio player can then use the sample time to schedule those events within the audio stream.
///
/// RhythmIter impls typically will use a [`EventIter`] to produce one or
/// multiple note or parameter change events. The event iter impl is an iterator too, so the
/// emitted content may dynamically change over time as well.
///
/// Iter item tuple args are: `(sample_time, optional_event, event_duration_in_samples)`.
pub trait RhythmIter: Debug {
    /// Create a sample time display printer, which serializes the given sample time to the emitter's
    /// time base as appropriated (in seconds or beats). May be useful for debugging purposes.
    fn sample_time_display(&self) -> Box<dyn SampleTimeDisplay>;

    /// Custom sample offset value which is applied to all emitted events.
    fn sample_offset(&self) -> SampleTime;
    /// Set a new custom sample offset value.
    fn set_sample_offset(&mut self, sample_offset: SampleTime);

    /// Step iter: runs pattern iter to generate a new pulse and then generate an event from
    /// the event iter.  
    fn run(&mut self) -> Option<(SampleTime, Option<Event>, SampleTime)>;

    /// Sample time iter: returns self.run() up and until the given sample time is reached.
    fn run_until_time(
        &mut self,
        sample_time: SampleTime,
    ) -> Option<(SampleTime, Option<Event>, SampleTime)>;
}

// -------------------------------------------------------------------------------------------------

/// Standard iterator impl for RhythmIter.
impl Iterator for dyn RhythmIter {
    type Item = (SampleTime, Option<Event>, SampleTime);

    fn next(&mut self) -> Option<Self::Item> {
        self.run()
    }
}

// -------------------------------------------------------------------------------------------------

/// A `Rhythm` is a resettable, dyn clonable `RhythmIter` with optional instrument and
/// time base setters.
///
/// Rhythms can be reset and cloned (duplicated), so that they can be triggered multiple times
/// using possibly different patterns and time bases.
pub trait Rhythm: RhythmIter {
    /// Length in samples of a single step in the rhythm's internal pattern.
    fn pattern_step_length(&self) -> f64;
    /// Get length in steps of the rhythm's internal pattern (cycle length in steps).
    /// A rhythm pattern repeats after self.pattern_step_length() * self.pattern_length() samples.
    fn pattern_length(&self) -> usize;

    /// Set or update the rhythm's internal beat or second time bases with a new time base.
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
