use crate::{
    events::{PatternEvent, PatternEventIter},
    time::{BeatTimeBase, BeatTimeStep},
    Pattern, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(PatternEvent)` every nth [`BeatTimeStep`]::Beat or Bar.
pub struct BeatTimePattern {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    current_counter: u32,
    current_sample_time: f64,
    event_iter: Box<dyn PatternEventIter>,
}

impl BeatTimePattern {
    /// Create a new beat time pattern which emits the given `value` every beat-time `step`.  
    pub fn new<Iter: PatternEventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        event_iter: Iter,
    ) -> Self {
        Self::new_with_offset(time_base, step, BeatTimeStep::Beats(0), event_iter)
    }

    /// Create a new beat time pattern which emits the given `value` every beat-time `step`
    /// starting at the given beat-time `offset`.  
    pub fn new_with_offset<Iter: PatternEventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        event_iter: Iter,
    ) -> Self {
        let current_sample_time = offset.to_samples(&time_base);
        let current_counter = 0;
        Self {
            time_base,
            step,
            offset,
            current_counter,
            current_sample_time,
            event_iter: Box::new(event_iter),
        }
    }
}

impl Iterator for BeatTimePattern {
    type Item = (SampleTime, Option<PatternEvent>);

    fn next(&mut self) -> Option<Self::Item> {
        // fetch current value
        let sample_time = self.current_sample_time as SampleTime;
        // move sample_time and counter
        let step = match self.step {
            BeatTimeStep::Beats(step) => step,
            BeatTimeStep::Bar(step) => step,
        };
        let value: Option<Self::Item> = if self.current_counter == 0 {
            Some((sample_time, self.event_iter.next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        match self.step {
            BeatTimeStep::Beats(_) => {
                self.current_sample_time += self.time_base.samples_per_beat() as f64;
            }
            BeatTimeStep::Bar(_) => {
                self.current_sample_time +=
                    self.time_base.samples_per_beat() as f64 * self.time_base.beats_per_bar as f64;
            }
        };
        // move counter
        self.current_counter += 1;
        if self.current_counter == step {
            self.current_counter = 0;
        }
        value
    }
}

impl Pattern for BeatTimePattern {
    fn current_event(&self) -> &dyn PatternEventIter {
        &*self.event_iter
    }
    fn current_sample_time(&self) -> SampleTime {
        self.current_sample_time as SampleTime
    }

    fn reset(&mut self) {
        self.event_iter.reset();
        self.current_sample_time = self.offset.to_samples(&self.time_base);
        self.current_counter = 0;
    }
}
