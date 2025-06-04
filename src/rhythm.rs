//! Periodically emit `Events` via an `EventIter` with a given time base on a
//! rhythmical pattern defined via a `Pattern`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{BeatTimeBase, Event, InputParameter, SampleTime};

// -------------------------------------------------------------------------------------------------

pub(crate) mod generic;

pub mod beat_time;
pub mod second_time;

// -------------------------------------------------------------------------------------------------

/// Iter item as produced by [`RhythmIter`]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RhythmIterItem {
    pub time: SampleTime,
    pub event: Option<Event>,
    pub duration: SampleTime,
}

impl RhythmIterItem {
    /// Create a new `RhythmIterItem` with the given sample offset
    /// added to the iter items sample time.
    #[must_use]
    pub fn with_offset(self, offset: SampleTime) -> Self {
        Self {
            time: self.time + offset,
            ..self
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A refcounted function which may transform emitted [`Event`] contents of a [`RhythmIter`].
pub type RhythmEventTransform = Rc<dyn Fn(&mut Event)>;

// -------------------------------------------------------------------------------------------------

/// A `RhythmIter` is an iterator which emits sample time tagged optional [`Event`]
/// items.
///
/// It triggers events periodically, producing events at specific sample times with specific
/// pulse durations. An audio player can use the sample time to schedule those events within the
/// audio stream.
///
/// `RhythmIter` impls typically will use a [`EventIter`](crate::EventIter) to produce one or
/// multiple notes, or a single parameter change event. The event iter impl is an iterator too,
/// so the emitted event content may dynamically change over time as well.
pub trait RhythmIter: Debug {
    /// Custom sample offset value which is applied to all emitted events.
    fn sample_offset(&self) -> SampleTime;
    /// Set a new custom sample offset value.
    fn set_sample_offset(&mut self, sample_offset: SampleTime);

    /// Get optional dynamic event transform function for the iterator
    fn event_transform(&self) -> &Option<RhythmEventTransform>;
    /// Set an optional dynamic event transform function for the iterator, which gets invoked for
    /// every emitted event. Note that transforms can not change event times, but only event content.
    fn set_event_transform(&mut self, transform: Option<RhythmEventTransform>);

    /// Sample time iter: Generate a single next due event but running the pattern to generate a new
    /// pulse, if the pulse's sample time is smaller than the given sample time. Then generates an
    /// event from the event iter and returns it.
    ///
    /// Returns `None` when no event is due of when the pattern finished playing, else Some event.
    fn run_until_time(&mut self, sample_time: SampleTime) -> Option<RhythmIterItem>;

    /// Skip *all events* until the given target time is reached.
    ///
    /// This calls `run_until_time` by default, until the target time is reached and
    /// discards all generated events, but may be overridden to optimize run time.
    fn advance_until_time(&mut self, sample_time: SampleTime) {
        while self.run_until_time(sample_time).is_some() {
            // continue
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Standard iterator impl for [`RhythmIter`].
impl Iterator for dyn RhythmIter {
    type Item = RhythmIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.run_until_time(SampleTime::MAX)
    }
}

// -------------------------------------------------------------------------------------------------

/// A `Rhythm` is a resettable, dyn clonable `RhythmIter` with a beat time base.
///
/// Rhythms can be reset and cloned (duplicated), so that they can be triggered multiple times
/// using possibly different patterns and time bases.
pub trait Rhythm: RhythmIter {
    /// Shared access to the rhythms input parameters, if any.
    fn input_parameters(&self) -> &[Rc<RefCell<InputParameter>>];

    /// Length in samples of a single step in the rhythm's internal pattern.
    fn pattern_step_length(&self) -> f64;
    /// Get length in steps of the rhythm's internal pattern (cycle length in steps).
    /// A rhythm pattern repeats after `self.pattern_step_length() * self.pattern_length()` samples.
    fn pattern_length(&self) -> usize;

    /// Get the rhythmiters internal beat time base.
    fn time_base(&self) -> &BeatTimeBase;
    /// Update the rhythm iters beat time bases with a new time base (e.g. on tempo changes).
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Set event which triggered, started the rhythm.
    fn set_trigger_event(&mut self, trigger: &Event);

    /// Create a new cloned instance of this rhythm. This actually is a clone(), wrapped into
    /// a `Box<dyn Rhythm>`, but called 'duplicate' to avoid conflicts with possible Clone impls.
    fn duplicate(&self) -> Rc<RefCell<dyn Rhythm>>;
    /// Resets/rewinds the rhythm to its initial state.
    fn reset(&mut self);
}
