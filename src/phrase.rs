use std::cell::RefCell;

use crate::{event::Event, Rhythm, SampleTime};

// -------------------------------------------------------------------------------------------------

/// Combines and runs one or more [`Rhythm`] at the same time, allowing to form more complex
/// patterns that are meant to run together.
///
/// An example phrase is a drum-kit pattern where each instrument's pattern is defined separately
/// and then is combined into a single "big" pattern to play the entire kit together.
pub struct Phrase {
    patterns: Vec<Box<RefCell<dyn Rhythm>>>,
    held_back_events: Vec<(SampleTime, Option<Event>)>,
}

impl Phrase {
    pub fn new(patterns: Vec<Box<RefCell<dyn Rhythm>>>) -> Self {
        Self {
            patterns,
            held_back_events: Vec::new(),
        }
    }

    /// Run all emitters in the sequence until a given sample time is reached and call given
    /// visitor function for all emitted events.
    pub fn run_until_time<F>(&mut self, run_sample_time: SampleTime, mut visitor: F)
    where
        F: FnMut(SampleTime, &Option<Event>),
    {
        // emit held back events first
        for (sample_time, event) in &self.held_back_events {
            if *sample_time < run_sample_time {
                visitor(*sample_time, event);
            }
        }
        self.held_back_events
            .retain(|(sample_time, _)| *sample_time >= run_sample_time);
        // then all new ones
        for pattern in self.patterns.iter_mut() {
            let pattern = pattern.get_mut();
            if pattern.current_sample_time() < run_sample_time {
                for (sample_time, event) in pattern {
                    if sample_time >= run_sample_time {
                        // hold last overshooted event back
                        self.held_back_events.push((sample_time, event));
                        break;
                    }
                    visitor(sample_time, &event);
                }
            }
        }
    }
}
