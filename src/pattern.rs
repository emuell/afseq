//! Rhythmical pattern as sequence of pulses in a `Rhythm`.

use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{BeatTimeBase, PulseStepTime, PulseValue};

pub mod empty;
pub mod euclidean;
pub mod fixed;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Interface for a pulse pattern generator as used by [Rhythm](`crate::Rhythm`).
pub trait Pattern: Debug {
    /// Returns if there is a valid pattern. If empty, it can't be run.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Return number of steps this pattern has. The pattern repeats after this. When the size
    /// is unknown, e.g. in external callbacks that generated pulses, 0 is returned, but
    /// self.is_empty will still be true.
    fn len(&self) -> usize;

    /// Run and move the pattern by a single step. Returns the pulse value in range `[0, 1]` and
    /// the sub division step the pattern moved along, also in range `[0, 1]`.
    /// A pulse value of 1 means that an event should be emitted and 0 that no event should be
    /// emitted. Values inbetween 0 and 1 may be treated as probablilities or get clamped,
    /// depending on the rhythm impl which is using the pattern.
    fn run(&mut self) -> (PulseValue, PulseStepTime);

    /// Set or update the pattern's internal beat or second time base with the new time base.
    /// Note: SampleTimeBase can be derived from BeatTimeBase via `SecondTimeBase::from(beat_time)`
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Create a new cloned instance of this event iter. This actualy is a clone(), wrapped into
    /// a `Rc<RefCell<dyn EventIter>>`, but called 'duplicate' to avoid conflicts with possible
    /// Clone impls.
    fn duplicate(&self) -> Rc<RefCell<dyn Pattern>>;

    /// Reset the pattern genertor, so it emits the same values as if it was freshly initialized.
    /// This does to reset the pattern itself, but onlt the pattern playback position.
    fn reset(&mut self);
}
