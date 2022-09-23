//! Periodically emit `Events` via an `EventIter` on a given time base.

use crate::{
    event::{Event, EventIter},
    SampleTime,
};

pub mod beat_time;
pub mod beat_time_sequence;

// -------------------------------------------------------------------------------------------------

/// A `Rhythm` is an iterator which emits optional [`Event`] in sample-rate resolution.
///
/// A `Rhythm` is what triggers events rythmically or periodically, producing events that happen
/// at a specific sample time. An audio players will use the sample time to schedule those events
/// within the audio stream.
///
/// Rhythm impls will typically use a [`EventIter`] to produce note or parameter change
/// events, so all emitted events are fetched from some iterator as well and thus may dynamically
/// change over time as well.
pub trait Rhythm: Iterator<Item = (SampleTime, Option<Event>)> {
    /// Access to the emitters current [`EventIter`] state.
    fn current_event(&self) -> &dyn EventIter;
    /// Access to the emitters current sample time offset.
    fn current_sample_time(&self) -> SampleTime;

    /// Resets/rewinds the pattern iterator to its initial state.
    fn reset(&mut self);
}
