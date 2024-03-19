//! Rhythmical pattern as sequence of pulses in a `Rhythm`.

use std::{borrow::Cow, fmt::Debug};

use crate::{BeatTimeBase, PulseIterItem};

pub mod empty;
pub mod euclidean;
pub mod fixed;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Interface for a pulse pattern iterator as used by [Rhythm](`crate::Rhythm`).
pub trait Pattern: Debug {
    /// Returns if there is a valid pattern. If empty, it can't be run.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Return number of pulses this pattern has. The pattern repeats after this. When the size
    /// is unknown, e.g. in external callbacks that generated pulses, 0 is returned, but
    /// `self.is_empty` will still be true.
    fn len(&self) -> usize;

    /// Run and move the pattern by a single step and return the to emitted pulse.
    /// When None, the pattern finished playback.
    fn run(&mut self) -> Option<PulseIterItem>;

    /// Set or update the pattern's internal beat or second time base with the new time base.
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Set optional, application specific external context data for the pattern.
    fn set_external_context(&mut self, data: &[(Cow<str>, f64)]);

    /// Set how many times the pattern should be repeated. If 0, the pattern will be run once.
    /// When None, which is the default, the pattern will be repeated indefinitely.
    fn set_repeat_count(&mut self, count: Option<usize>);

    /// Create a new cloned instance of this event iter. This actualy is a clone(), wrapped into
    /// a `Box<dyn EventIter>`, but called 'duplicate' to avoid conflicts with possible
    /// Clone impls.
    fn duplicate(&self) -> Box<dyn Pattern>;

    /// Reset the pattern genertor, so it emits the same values as if it was freshly initialized.
    /// This does to reset the pattern itself, but onlt the pattern playback position.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// Standard Iterator impl for Pattern.
impl Iterator for dyn Pattern {
    type Item = PulseIterItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.run()
    }
}
