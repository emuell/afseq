use crate::{
    event::{Event, EventIter},
    time::{SecondTimeBase, SecondTimeStep},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(Event)` every nth [`SecondStep`]::Beat or Bar.
pub struct SecondTimeRhythm {
    time_base: SecondTimeBase,
    step: SecondTimeStep,
    offset: SecondTimeStep,
    current_sample_time: f64,
    event_iter: Box<dyn EventIter>,
}

impl SecondTimeRhythm {
    /// Create a new beat time pattern which emits the given `value` every beat-time `step`.  
    pub fn new<Iter: EventIter + 'static>(
        time_base: SecondTimeBase,
        step: SecondTimeStep,
        event_iter: Iter,
    ) -> Self {
        Self::new_with_offset(time_base, step, 0.0, event_iter)
    }

    /// Create a new beat time pattern which emits the given `value` every beat-time `step`
    /// starting at the given beat-time `offset`.  
    pub fn new_with_offset<Iter: EventIter + 'static>(
        time_base: SecondTimeBase,
        step: SecondTimeStep,
        offset: SecondTimeStep,
        event_iter: Iter,
    ) -> Self {
        let current_sample_time = time_base.seconds_to_samples(offset) as f64;
        Self {
            time_base,
            step,
            offset,
            current_sample_time,
            event_iter: Box::new(event_iter),
        }
    }
}

impl Iterator for SecondTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        let sample_time = self.current_sample_time as SampleTime;
        self.current_sample_time += self.time_base.seconds_to_samples(self.step) as f64;
        Some((sample_time, self.event_iter.next()))
    }
}

impl Rhythm for SecondTimeRhythm {
    fn reset(&mut self) {
        self.event_iter.reset();
        self.current_sample_time = self.time_base.seconds_to_samples(self.offset) as f64;
    }
}
