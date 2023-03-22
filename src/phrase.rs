//! Combine multiple `Rythm` iterators into a single one to play them at the same time.

use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    rc::Rc,
};

use crate::{
    event::Event, prelude::BeatTimeStep, time::SampleTimeDisplay, BeatTimeBase, Rhythm,
    SampleOffset, SampleTime,
};

#[cfg(doc)]
use crate::Sequence;

// -------------------------------------------------------------------------------------------------

type RhythmIndex = usize;

/// A single slot in a [`Phrase`] vector.
pub enum RhythmSlot {
    /// Stop previous playing rhythm and play nothing new.
    /// This can be useful in [`Sequence`] to create empty placeholder slots.
    Stop,
    /// TODO: Continue playing previous playing rhythm.
    /// This is only meaningful in [`Sequence`] rhythms to repeat a rhythm instead
    /// of restarting it at a new sequence position.
    Continue,
    /// Play the given boxed rhytm in this slot.
    Rhythm(Box<dyn Rhythm>),
}

impl RhythmSlot {
    /// Create a new RhythmSlot from an unboxed [`Rhythm`] instance.
    pub fn from_rhythm<R>(rhythm: R) -> Self
    where
        R: Rhythm + 'static,
    {
        Self::Rhythm(Box::new(rhythm))
    }
    /// Create a new RhythmSlot from a boxed [`Rhythm`] instance.
    pub fn from_boxed_rhythm(rhythm: Box<dyn Rhythm>) -> Self {
        Self::Rhythm(rhythm)
    }
}

/// Convert an unboxed rhythm to a RhythmSlot
impl<R> From<R> for RhythmSlot
where
    R: Rhythm + 'static,
{
    fn from(r: R) -> RhythmSlot {
        RhythmSlot::Rhythm(Box::new(r))
    }
}

/// Convert an boxed rhythm to a RhythmSlot
impl From<Box<dyn Rhythm>> for RhythmSlot {
    fn from(r: Box<dyn Rhythm>) -> RhythmSlot {
        RhythmSlot::Rhythm(r)
    }
}

// -------------------------------------------------------------------------------------------------

/// Combines multiple [`Rhythm`] into a new one, allowing to form more complex rhythms that are
/// meant to run together. Further it allows to run/evaluate rhythms until a specific sample time
/// is reached.
///
/// An example phrase is a drum-kit pattern where each instrument's pattern is defined separately
/// and then is combined into a single "big" pattern to play the entire kit together.
///
/// The `run_until_time` function is also used by [`Sequence`] to play a phrase with a player engine.
#[derive(Clone)]
pub struct Phrase {
    time_base: BeatTimeBase,
    offset: BeatTimeStep,
    sample_offset: SampleOffset,
    length: BeatTimeStep,
    rhythm_slots: Rc<RefCell<Vec<RhythmSlot>>>,
    next_events: Vec<Option<(RhythmIndex, SampleTime, Option<Event>)>>,
    held_back_event: Option<(RhythmIndex, SampleTime, Option<Event>)>,
}

impl Phrase {
    /// Create a new phrase from a vector of [`RhythmSlot`] and the given length.
    /// RhythmSlot has `Into` implementastions, so you can also pass a vector of 
    /// unboxed rhythm instance here.
    pub fn new<R: Into<RhythmSlot>>(
        time_base: BeatTimeBase,
        rhythm_slots: Vec<R>,
        length: BeatTimeStep,
    ) -> Self {
        let offset = BeatTimeStep::Beats(0.0);
        let sample_offset = 0;
        let next_events = vec![None; rhythm_slots.len()];
        let held_back_event = None;
        Self {
            time_base,
            offset,
            sample_offset,
            length,
            rhythm_slots: Rc::new(RefCell::new(
                rhythm_slots
                    .into_iter()
                    .map(|rhythm| rhythm.into())
                    .collect::<Vec<_>>(),
            )),
            next_events,
            held_back_event,
        }
    }

    /// Apply the given beat-time step offset to all events.
    pub fn with_offset<O: Into<Option<BeatTimeStep>>>(&self, offset: O) -> Phrase {
        Self {
            time_base: self.time_base,
            offset: offset.into().unwrap_or(BeatTimeStep::Beats(0.0)),
            sample_offset: self.sample_offset,
            length: self.length,
            rhythm_slots: self.rhythm_slots.clone(),
            next_events: self.next_events.clone(),
            held_back_event: self.held_back_event.clone(),
        }
    }

    /// Read-only access to our phrase length.
    /// This is only applied in [`Sequence`].
    pub fn length(&self) -> BeatTimeStep {
        self.length
    }

    /// Read-only access to our rhythm slots.
    pub fn rhythms(&self) -> Ref<Vec<RhythmSlot>> {
        self.rhythm_slots.borrow()
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn run_until_time<F>(&mut self, run_until_time: SampleTime, consumer: &mut F)
    where
        F: FnMut(RhythmIndex, SampleTime, &Option<Event>),
    {
        // emit last held back event first
        if let Some((rhythm_index, sample_time, event)) = &self.held_back_event {
            if *sample_time < run_until_time {
                consumer(*rhythm_index, *sample_time, event);
                self.held_back_event = None;
            } else {
                // held back event is not yet due
                return;
            }
        }
        // then emit next events until we've reached the desired sample_time
        while let Some((rhythm_index, sample_time, event)) = self.next_event() {
            if sample_time >= run_until_time {
                // hold this event back for the next run
                self.held_back_event = Some((rhythm_index, sample_time, event));
                break;
            }
            consumer(rhythm_index, sample_time, &event);
        }
    }

    fn next_event(&mut self) -> Option<(RhythmIndex, SampleTime, Option<Event>)> {
        // fetch next events in all rhythms
        for (rhythm_index, (rhythm_slot, next_event)) in self
            .rhythm_slots
            .borrow_mut()
            .iter_mut()
            .zip(self.next_events.iter_mut())
            .enumerate()
        {
            if !next_event.is_some() {
                match rhythm_slot {
                    RhythmSlot::Stop => *next_event = None,
                    RhythmSlot::Continue => todo!(),
                    RhythmSlot::Rhythm(rhythm) => {
                        if let Some((sample_time, event)) = rhythm.next() {
                            *next_event = Some((rhythm_index, sample_time, event));
                        } else {
                            *next_event = None;
                        }
                    }
                }
            }
        }
        // select the next from all pre-fetched events with the smallest sample time
        let next_due = self.next_events.iter_mut().reduce(|min, next| {
            if let Some((_, min_sample_time, _)) = min {
                if let Some((_, next_sample_time, _)) = next {
                    match min_sample_time.cmp(&next_sample_time) {
                        Ordering::Less | Ordering::Equal => min,
                        Ordering::Greater => next,
                    }
                } else {
                    min
                }
            } else {
                next
            }
        });
        if let Some(next_due) = next_due {
            let next = next_due.clone();
            *next_due = None; // consume
            if let Some((rhythm_index, sample_time, event)) = next {
                let sample_offset = (self.sample_offset
                    + self.offset.to_samples(&self.time_base) as i64)
                    .max(0) as u64;
                Some((rhythm_index, sample_offset + sample_time, event))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Iterator for Phrase {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, sample_time, event)) = self.next_event() {
            Some((sample_time, event))
        } else {
            None
        }
    }
}

impl Rhythm for Phrase {
    fn time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }

    fn sample_offset(&self) -> SampleOffset {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleOffset) {
        self.sample_offset = sample_offset
    }

    fn reset(&mut self) {
        // reset our own iter state
        self.next_events.fill(None);
        self.held_back_event = None;
        // reset all our rhythm iters
        for rhythm_slot in self.rhythm_slots.borrow_mut().iter_mut() {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.reset()
            }
        }
    }
}
