//! Event iterator which generates `Events` within a `Pattern`.

use std::fmt::Debug;

use crate::{BeatTimeBase, Event, ParameterSet, RhythmEvent};

type Fraction = num_rational::Rational32;

// -------------------------------------------------------------------------------------------------

pub mod cycle;
pub mod empty;
pub mod fixed;
pub mod mutated;
#[cfg(feature = "scripting")]
pub mod scripted;
#[cfg(feature = "scripting")]
pub mod scripted_cycle;

pub use fixed::{
    new_empty_note_emitter, new_note_emitter, new_note_sequence_emitter,
    new_parameter_change_emitter, new_polyphonic_note_emitter,
    new_polyphonic_note_sequence_emitter,
};

// -------------------------------------------------------------------------------------------------

/// Event with start and length fraction as produced by [`Emitter`].
#[derive(Clone, PartialEq, Debug)]
pub struct EmitterEvent {
    pub event: Event,     // The emitted event
    pub start: Fraction,  // Relative event start time in range 0..1
    pub length: Fraction, // Relative event length in range 0..1
}

impl EmitterEvent {
    /// Create a new emitter event with a default start and length.
    pub fn new(event: Event) -> Self {
        Self {
            event,
            start: Fraction::ZERO,
            length: Fraction::ONE,
        }
    }

    /// Create a new emitter event with custom start and length fractions.
    pub fn new_with_fraction(event: Event, start: Fraction, length: Fraction) -> Self {
        Self {
            start,
            length,
            event,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// A clonable, resettable [`Event`] iterator.
///
/// Used by [`Pattern`](crate::Pattern) to generate events from pulse rhythms.
pub trait Emitter: Debug {
    /// Update the iterator's internal beat time base with the new time base.
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Set optional event which triggered, started the iter, if any.
    fn set_trigger_event(&mut self, event: &Event);

    /// Set or update optional parameter map for callbacks.
    fn set_parameters(&mut self, parameters: ParameterSet);

    /// Move iterator with the given rhythm event pulse value forward.
    /// `pulse` contains the current value and timing information for the current step in the pattern.
    /// `emit_event` indicates whether the iterator should trigger the next event in the sequence as
    /// evaluated by the pattern's gate.
    ///
    /// Returns an optional stack of emitter events for the given pulse value.
    fn run(&mut self, pulse: RhythmEvent, emit_event: bool) -> Option<Vec<EmitterEvent>>;

    /// Move iterator with the given pulse value forward without emitting events.
    ///
    /// This can be used to optimize iterator skipping in some Emitter implementations, but by
    /// default calls `run` and simply discards the generated events.
    fn advance(&mut self, pulse: RhythmEvent, emit_event: bool) {
        let _ = self.run(pulse, emit_event);
    }

    /// Create a new cloned instance of this emitter. This actually is a clone(), wrapped into
    /// a `Box<dyn Emitter>`, but called 'duplicate' to avoid conflicts with possible
    /// Clone impls.
    fn duplicate(&self) -> Box<dyn Emitter>;

    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// Standard Iterator impl for [`Emitter`].
///
/// Runs the emitter with a 1 valued [`RhythmEvent`].
impl Iterator for dyn Emitter {
    type Item = Vec<EmitterEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        let pulse = RhythmEvent {
            value: 1.0,
            step_time: 1.0,
        };
        let emit_event = true;
        self.run(pulse, emit_event)
    }
}
