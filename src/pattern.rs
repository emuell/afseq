use crate::{
    events::{PatternEvent, PatternEventIter},
    SampleTime,
};

pub mod beat_time;
pub mod beat_time_sequence;

// -------------------------------------------------------------------------------------------------

/// A `Pattern` is an iterator which emits optional [`PatternEvent`] events in sample-rate
/// resolution.
///
/// A `Pattern` is what triggers events rythmically or periodically, producing events that happen
/// at a specific sample time. An audio players will use the sample time to schedule those events
/// within the audio stream.
///
/// Pattern impls will typically use a [`PatternEventIter`] to produce note or parameter change
/// events, so all emitted events are fetched from some iterator as well and thus may dynamically
/// change over time as well.
pub trait Pattern: Iterator<Item = (SampleTime, Option<PatternEvent>)> {
    /// Access to the emitters current [`PatternEventIter`] state.
    fn current_event(&self) -> &dyn PatternEventIter;
    /// Access to the emitters current sample time offset.
    fn current_sample_time(&self) -> SampleTime;

    /// Resets/rewinds the pattern iterator to its initial state.
    fn reset(&mut self);
}
