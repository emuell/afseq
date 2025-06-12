//! Rhythmical pattern as sequence of pulses in a `Pattern`.

use std::fmt::Debug;

use crate::{BeatTimeBase, Event, ParameterSet, Pulse};

pub mod empty;
pub mod euclidean;
pub mod fixed;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Iterator item as produced by [`Rhythm`]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RhythmEvent {
    /// Pulse value in a rhythm in range \[0 - 1\] where 0 means don't trigger anything, and 1
    /// means definitely do trigger something. Values within 0 - 1 maybe trigger, depending on the
    /// pattern/rhythm/gate impl.
    pub value: f32,
    /// Pulse step time fraction in range \[0 - 1\]. 1 means advance by a full step, 0.5 means
    /// advance by a half step, etc.
    pub step_time: f64,
}

impl Default for RhythmEvent {
    fn default() -> Self {
        Self {
            value: 0.0,
            step_time: 1.0,
        }
    }
}

// -------------------------------------------------------------------------------------------------

/// Iterator for [`Pulse`], recursively flattening all subdivisions.
#[derive(Clone, Debug)]
pub struct RhythmEventIterator {
    values: Vec<RhythmEvent>,
    step: usize,
}

impl RhythmEventIterator {
    pub fn new(pulse: &Pulse) -> Self {
        let values = pulse.to_rhythm_events();
        let step = 0;
        Self { values, step }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn step(&self) -> usize {
        self.step
    }
}

impl Iterator for RhythmEventIterator {
    type Item = RhythmEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step >= self.values.len() {
            None
        } else {
            let event = self.values[self.step];
            self.step += 1;
            Some(event)
        }
    }
}

impl Pulse {
    /// Returns a flattened copy of all contained pulse values as events.
    pub fn to_rhythm_events(&self) -> Vec<RhythmEvent> {
        let mut values = vec![];
        self.expand_into(&mut values, 1.0);
        values
    }

    fn expand_into(&self, result: &mut Vec<RhythmEvent>, step_time: f64) {
        match self {
            Pulse::Pulse(value) => {
                let value = *value;
                result.push(RhythmEvent { value, step_time });
            }
            Pulse::SubDivision(ref sub_pulses) => {
                for sub_pulse in sub_pulses {
                    let sub_step_time = step_time / sub_pulses.len() as f64;
                    sub_pulse.expand_into(result, sub_step_time);
                }
            }
        }
    }
}

impl IntoIterator for Pulse {
    type Item = RhythmEvent;
    type IntoIter = RhythmEventIterator;

    fn into_iter(self) -> Self::IntoIter {
        RhythmEventIterator::new(&self)
    }
}

// -------------------------------------------------------------------------------------------------

/// Defines the rhythmic structure of a [`Pattern`](crate::Pattern).
pub trait Rhythm: Debug {
    /// Returns true if there is no rhythm to run.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Return number of pulses this rhythm has. The rhythm's pattern repeats after this count.
    /// When the size is unknown, e.g. in external callbacks that generated pulses, 0 is returned,
    /// but `self.is_empty` will still be false.
    fn len(&self) -> usize;

    /// Run and move the rhythm by a single step and return the emitted pulse event.
    /// When None, playback finished.
    fn run(&mut self) -> Option<RhythmEvent>;

    /// Set or update the rhythm's internal beat time base with the given new time base.
    fn set_time_base(&mut self, time_base: &BeatTimeBase);

    /// Set optional event which triggered, started the iter, if any, before running.
    fn set_trigger_event(&mut self, event: &Event);

    /// Set or update and optional parameter map for callbacks.
    fn set_parameters(&mut self, parameters: ParameterSet);

    /// Set how many times the rhythm pattern should be repeated. If 0, the rhythm will be run
    /// once. When None, which is the default, the rhythm will be repeated indefinitely.
    fn set_repeat_count(&mut self, count: Option<usize>);

    /// Create a new cloned instance of this rhythm. This actualy is a clone(), wrapped into
    /// a `Box<dyn Rhythm>`, but called 'duplicate' to avoid conflicts with possible Clone impls.
    fn duplicate(&self) -> Box<dyn Rhythm>;

    /// Reset the rhythm, so it emits the same values as if it was freshly initialized.
    /// This usually will only reset rhythm playback positions.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// Standard Iterator impl for Pattern.
impl Iterator for dyn Rhythm {
    type Item = RhythmEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.run()
    }
}
