use super::{BeatTimeBase, BeatTimeStep};
use crate::{Emitter, EmitterEvent, EmitterValue, SampleTime};

// -------------------------------------------------------------------------------------------------

/// Emits Some<EmitterValue> every nth BeatTimeStep::Beats or BeatTimeStep::Bar.
pub struct BeatTimeEmitter {
    time_base: BeatTimeBase,
    beat_time: BeatTimeStep,
    counter: u32,
    sample_time: f64,
    value: Box<dyn EmitterValue>,
}

impl BeatTimeEmitter {
    pub fn new<Value: EmitterValue + 'static>(
        time_base: BeatTimeBase,
        beat_time: BeatTimeStep,
        value: Value,
    ) -> Self {
        let sample_time = 0.0;
        let counter = 0;
        Self {
            time_base,
            beat_time,
            counter,
            sample_time,
            value: Box::new(value),
        }
    }
}

impl Iterator for BeatTimeEmitter {
    type Item = (SampleTime, Option<EmitterEvent>);

    fn next(&mut self) -> Option<Self::Item> {
        // fetch current value
        let sample_time = self.sample_time as SampleTime;
        // move sample_time and counter
        let step = match self.beat_time {
            BeatTimeStep::Beats(step) => step,
            BeatTimeStep::Bar(step) => step,
        };
        let value: Option<Self::Item> = if self.counter == 0 {
            Some((sample_time, self.value.next()))
        } else {
            Some((sample_time, None))
        };
        // move sample_time
        match self.beat_time {
            BeatTimeStep::Beats(_) => {
                self.sample_time += self.time_base.samples_per_beat() as f64;
            }
            BeatTimeStep::Bar(_) => {
                self.sample_time +=
                    self.time_base.samples_per_beat() as f64 * self.time_base.beats_per_bar as f64;
            }
        };
        // move counter
        self.counter += 1;
        if self.counter == step {
            self.counter = 0;
        }
        value
    }
}

impl Emitter for BeatTimeEmitter {
    fn current_value(&self) -> &dyn EmitterValue {
        &*self.value
    }
    fn current_sample_time(&self) -> SampleTime {
        self.sample_time as SampleTime
    }

    fn reset(&mut self) {
        self.value.reset();
        self.counter = 0;
        self.sample_time = 0.0;
    }
}
