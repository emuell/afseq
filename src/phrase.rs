//! Combine multiple `Rythm` iterators into a single one.

use std::{cell::RefCell, rc::Rc};

use crate::{event::Event, prelude::BeatTimeStep, BeatTimeBase, Rhythm, SampleTime};

// -------------------------------------------------------------------------------------------------

/// Combines multiple [`Rhythm`] into a new one, allowing to form more complex rhythms that are
/// meant to run together. Further it allows to run/evaluate rhythms, until a specific sample time
/// is reached.
///
/// An example phrase is a drum-kit pattern where each instrument's pattern is defined separately
/// and then is combined into a single "big" pattern to play the entire kit together.
///
/// The `run_until_time` function can then be used to feed the entire phrase into a player engine.
#[derive(Clone)]
pub struct Phrase {
    time_base: BeatTimeBase,
    offset: BeatTimeStep,
    rhythms: Rc<RefCell<Vec<Box<dyn Rhythm>>>>,
    next_events: Vec<Option<(SampleTime, Option<Event>)>>,
    held_back_event: Option<(SampleTime, Option<Event>)>,
}

impl Phrase {
    /// Create a new phrase from a vector of boxed `Rhythms`.
    pub fn new(time_base: BeatTimeBase, rhythms: Vec<Box<dyn Rhythm>>) -> Self {
        let offset = BeatTimeStep::Beats(0.0);
        let next_events = vec![None; rhythms.len()];
        let held_back_event = None;
        Self {
            time_base,
            offset,
            rhythms: Rc::new(RefCell::new(rhythms)),
            next_events,
            held_back_event,
        }
    }

    /// Apply the given beat-time step offset to all events.
    pub fn with_offset<O: Into<Option<BeatTimeStep>>>(&self, offset: O) -> Phrase {
        Self {
            time_base: self.time_base,
            offset: offset.into().unwrap_or(BeatTimeStep::Beats(0.0)),
            rhythms: self.rhythms.clone(),
            next_events: self.next_events.clone(),
            held_back_event: self.held_back_event.clone(),
        }
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn run_until_time<F>(&mut self, run_until_time: SampleTime, mut consumer: F)
    where
        F: FnMut(SampleTime, &Option<Event>),
    {
        // emit last held back event first
        if let Some((sample_time, event)) = &self.held_back_event {
            if *sample_time < run_until_time {
                consumer(*sample_time, event);
                self.held_back_event = None;
            }
        }
        // then emit next events until we've reached the desired sample_time
        while let Some((sample_time, event)) = self.next() {
            if sample_time >= run_until_time {
                // hold this event back for the next run
                self.held_back_event = Some((sample_time, event));
                break;
            }
            consumer(sample_time, &event);
        }
    }
}

impl Iterator for Phrase {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        // fetch next events in all rhythms
        for (rhythm, next_event) in self
            .rhythms
            .borrow_mut()
            .iter_mut()
            .zip(self.next_events.iter_mut())
        {
            if !next_event.is_some() {
                *next_event = rhythm.next();
            }
        }
        // select the next from all pre-fetched events with the smallest sample time
        let next_due = self.next_events.iter_mut().reduce(|min, next| {
            let (min_sample_time, _) = min.as_ref().unwrap();
            let (next_sample_time, _) = next.as_ref().unwrap();
            if *next_sample_time < *min_sample_time {
                next
            } else {
                min
            }
        });
        if let Some(next_due) = next_due {
            let next = next_due.clone();
            *next_due = None; // consume
            if let Some((sample_time, event)) = next {
                let sample_offset = self.offset.to_samples(&self.time_base) as u64;
                Some((sample_offset + sample_time, event))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Rhythm for Phrase {
    fn reset(&mut self) {
        // reset our own iter state
        self.next_events.fill(None);
        self.held_back_event = None;
        // reset all our rhythm iters
        for rhythm in self.rhythms.borrow_mut().iter_mut() {
            rhythm.reset();
        }
    }
}
