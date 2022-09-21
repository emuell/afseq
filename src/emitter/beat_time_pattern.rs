use super::{BeatTimeBase, BeatTimeStep};
use crate::{Emitter, EmitterEvent, EmitterValue, SampleTime};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(EmitterValue)` every BeatTimeStep time step, depending on a trigger pattern.
pub struct BeatTimePatternEmitter {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    pattern: Vec<bool>,
    pos_in_pattern: u32,
    sample_time: f64,
    value: Box<dyn EmitterValue>,
}

impl BeatTimePatternEmitter {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.
    pub fn new<Value: EmitterValue + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        pattern: Vec<bool>,
        value: Value,
    ) -> Self {
        let offset = BeatTimeStep::Beats(0);
        Self::new_with_offset(time_base, step, offset, pattern, value)
    }

    /// Create a new pattern based emitter which emits `value` every beat_time `step`
    /// starting at the given beat_time `offset`.  
    pub fn new_with_offset<Value: EmitterValue + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        pattern: Vec<bool>,
        value: Value,
    ) -> Self {
        let sample_time = offset.to_samples(&time_base);
        let pos_in_pattern = 0;
        Self {
            time_base,
            step,
            offset,
            pattern,
            pos_in_pattern,
            sample_time,
            value: Box::new(value),
        }
    }
}

impl Iterator for BeatTimePatternEmitter {
    type Item = (SampleTime, Option<EmitterEvent>);

    fn next(&mut self) -> Option<Self::Item> {
        // fetch current value
        let sample_time = self.sample_time as SampleTime;
        let value = if !self.pattern.is_empty() && self.pattern[self.pos_in_pattern as usize] {
            Some((sample_time, self.value.next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        match self.step {
            BeatTimeStep::Beats(step) => {
                self.sample_time += self.time_base.samples_per_beat() as f64 * step as f64;
            }
            BeatTimeStep::Bar(step) => {
                self.sample_time += self.time_base.samples_per_beat() as f64
                    * self.time_base.beats_per_bar as f64
                    * step as f64;
            }
        };
        // move pattern pos
        self.pos_in_pattern += 1;
        if self.pos_in_pattern >= self.pattern.len() as u32 {
            self.pos_in_pattern = 0;
        }
        // return previous value
        value
    }
}

impl Emitter for BeatTimePatternEmitter {
    fn current_value(&self) -> &dyn EmitterValue {
        &*self.value
    }
    fn current_sample_time(&self) -> SampleTime {
        self.sample_time as SampleTime
    }
    fn reset(&mut self) {
        self.value.reset();
        self.sample_time = self.offset.to_samples(&self.time_base);
        self.pos_in_pattern = 0;
    }
}
