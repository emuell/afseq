//! Rhythmical pattern as sequence of pulses in a `Rhythm`.

use std::fmt::Debug;

pub mod empty;
pub mod euclidean;
pub mod fixed;

// -------------------------------------------------------------------------------------------------

/// Interface for a pulse pattern generator as used by [`Rhythm`].
pub trait Pattern: Debug {
    /// Returns if there is a valid pattern. If empty, it can't be run.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return number of steps this pattern has. The pattern repeats after this. When the size 
    /// is unknown, e.g. in external callbacks that generated pulses, 0 is returned, but 
    /// self.is_empty will still be true.
    fn len(&self) -> usize;

    /// Run and move the pattern by a single step. Returns the pulse value in range [0, 1],
    /// where 1 means that an event should be emitted and 0 that no event should be emitted.
    /// Values inbetween 0 and 1 may be treated as probablilities or get clamped, depending on
    /// the rhythm impl which is using the pattern.   
    fn run(&mut self) -> f32;

    /// Reset the pattern genertor, so it emits the same values as if it was freshly initialized.
    // This does to reset the pattern itself, but onlt the pattern playback position.
    fn reset(&mut self);
}
