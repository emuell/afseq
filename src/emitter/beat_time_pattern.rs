use super::{BeatTimeBase, BeatTimeStep};
use crate::{Emitter, EmitterEvent, EmitterValue, SampleTime};

// -------------------------------------------------------------------------------------------------

/// Emits Some<EmitterValue> every BeatTimeStep time step, depending on a trigger pattern.
pub struct BeatTimePatternEmitter {
    time_base: BeatTimeBase,
    time_step: BeatTimeStep,
    pattern: Vec<bool>,
    pattern_pos: u32,
    sample_time: f64,
    value: Box<dyn EmitterValue>,
}

impl BeatTimePatternEmitter {
    pub fn new<Value: EmitterValue + 'static>(
        time_base: BeatTimeBase,
        time_step: BeatTimeStep,
        pattern: Vec<bool>,
        value: Value,
    ) -> Self {
        let sample_time = 0.0;
        let pattern_pos = 0;
        Self {
            time_base,
            time_step,
            pattern,
            pattern_pos,
            sample_time,
            value: Box::new(value),
        }
    }
}

impl Iterator for BeatTimePatternEmitter {
    type Item = (SampleTime, Option<Vec<EmitterEvent>>);

    fn next(&mut self) -> Option<Self::Item> {
        // fetch current value
        let sample_time = self.sample_time as SampleTime;
        let value = if !self.pattern.is_empty() && self.pattern[self.pattern_pos as usize] {
            Some((sample_time, self.value.next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        match self.time_step {
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
        self.pattern_pos += 1;
        if self.pattern_pos >= self.pattern.len() as u32 {
            self.pattern_pos = 0;
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
}
