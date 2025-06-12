use crate::{BeatTimeBase, Event, Gate, ParameterSet, RhythmEvent};

// -------------------------------------------------------------------------------------------------

/// Gate implementation which passes all pulse values > a specified threshold value (by default 0).
#[derive(Debug, Clone)]
pub struct ThresholdGate {
    threshold: f32,
}

impl ThresholdGate {
    pub fn new() -> Self {
        Self::with_threshold(0.0)
    }

    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Default for ThresholdGate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate for ThresholdGate {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_trigger_event(&mut self, _event: &Event) {
        // nothing to do
    }

    fn set_parameters(&mut self, _parameters: ParameterSet) {
        // nothing to do
    }

    fn run(&mut self, pulse: &RhythmEvent) -> bool {
        pulse.value > self.threshold
    }

    fn duplicate(&self) -> Box<dyn Gate> {
        Box::new(self.clone())
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
