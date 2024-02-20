//! Combine, stack multiple `Rythm` iterators into a single one, in order to play multiple
//! Rhythms at once.

use std::{cell::RefCell, cmp::Ordering, fmt::Debug, rc::Rc};

use crate::{
    event::Event, prelude::BeatTimeStep, time::SampleTimeDisplay, BeatTimeBase, Rhythm, SampleTime,
};

#[cfg(doc)]
use crate::Sequence;

// -------------------------------------------------------------------------------------------------

/// Rhythm index in `RhythmEvent`.
pub type RhythmIndex = usize;
/// Event as emitted by Phrase, tagged with a sample time and rhythm index.
pub type RhythmEvent = (RhythmIndex, SampleTime, Option<Event>);

// -------------------------------------------------------------------------------------------------

/// A single slot in a [`Phrase`] vector.
#[derive(Clone, Debug)]
pub enum RhythmSlot {
    /// Stop previous playing rhythm and/or simply play nothing.
    /// This can be useful to create empty placeholder slots in e.g. a [`Sequence`].
    Stop,
    /// Continue playing a previously played rhythm in a [`Sequence`].
    Continue,
    /// Play a shared rhytm in this slot.
    /// NB: This is a shared reference, in order to resolve 'Continue' modes in sequences.
    Rhythm(Rc<RefCell<dyn Rhythm>>),
}

/// Convert an unboxed Rhythm to a RhythmSlot
impl<R> From<R> for RhythmSlot
where
    R: Rhythm + 'static,
{
    fn from(rhythm: R) -> RhythmSlot {
        RhythmSlot::Rhythm(Rc::new(RefCell::new(rhythm)))
    }
}

/// Convert a shared Rhythm to a RhythmSlot
impl From<Rc<RefCell<dyn Rhythm>>> for RhythmSlot {
    fn from(rhythm: Rc<RefCell<dyn Rhythm>>) -> RhythmSlot {
        RhythmSlot::Rhythm(rhythm)
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
#[derive(Clone, Debug)]
pub struct Phrase {
    time_base: BeatTimeBase,
    length: BeatTimeStep,
    rhythm_slots: Vec<RhythmSlot>,
    next_events: Vec<Option<RhythmEvent>>,
    sample_offset: SampleTime,
}

impl Phrase {
    /// Create a new phrase from a vector of [`RhythmSlot`] and the given length.
    /// RhythmSlot has `Into` implementastions, so you can also pass a vector of
    /// boxed or raw rhythm instance here.
    pub fn new<R: Into<RhythmSlot>>(
        time_base: BeatTimeBase,
        rhythm_slots: Vec<R>,
        length: BeatTimeStep,
    ) -> Self {
        let next_events = vec![None; rhythm_slots.len()];
        let sample_offset = 0;
        Self {
            time_base,
            length,
            rhythm_slots: rhythm_slots
                .into_iter()
                .map(|rhythm| -> RhythmSlot { rhythm.into() })
                .collect::<Vec<_>>(),
            next_events,
            sample_offset,
        }
    }

    /// Read-only access to our phrase length.
    /// This is applied in [`Sequence`] only.
    pub fn length(&self) -> BeatTimeStep {
        self.length
    }

    /// Read-only access to our rhythm slots.
    pub fn rhythm_slots(&self) -> &Vec<RhythmSlot> {
        &self.rhythm_slots
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn run_until_time<F>(&mut self, run_until_time: SampleTime, consumer: &mut F)
    where
        F: FnMut(RhythmIndex, SampleTime, Option<Event>),
    {
        // emit next events until we've reached the desired sample_time
        while let Some((rhythm_index, sample_time, event)) =
            self.next_event_until_time(run_until_time)
        {
            assert!(sample_time < run_until_time);
            consumer(rhythm_index, sample_time, event);
        }
    }

    /// reset playback status and shift events to the given sample position.
    /// Further take over rhythms from the passed previously playing phrase for RhythmSlot::Continue slots.   
    pub fn reset_with_offset(&mut self, sample_offset: SampleTime, previous_phrase: &Phrase) {
        // reset rhythm iters, unless they are in continue mode. in contine mode, copy the slot
        // from the previously playing phrase and adjust sample offsets to fit.
        for rhythm_index in 0..self.rhythm_slots.len() {
            match &mut self.rhythm_slots[rhythm_index] {
                RhythmSlot::Rhythm(rhythm) => {
                    {
                        let mut rhythm = rhythm.borrow_mut();
                        rhythm.reset();
                        rhythm.set_sample_offset(sample_offset);
                    }
                    self.next_events[rhythm_index] = None;
                }
                RhythmSlot::Stop => {
                    self.next_events[rhythm_index] = None;
                }
                RhythmSlot::Continue => {
                    // take over pending events
                    self.next_events[rhythm_index] =
                        previous_phrase.next_events[rhythm_index].clone();
                    // take over rhythm
                    self.rhythm_slots[rhythm_index] =
                        previous_phrase.rhythm_slots[rhythm_index].clone();
                }
            }
        }
    }

    fn next_event_until_time(&mut self, sample_time: SampleTime) -> Option<RhythmEvent> {
        // fetch next events in all rhythms
        for (rhythm_index, (rhythm_slot, next_event)) in self
            .rhythm_slots
            .iter_mut()
            .zip(self.next_events.iter_mut())
            .enumerate()
        {
            if !next_event.is_some() {
                match rhythm_slot {
                    // NB: Continue mode is resolved by the Sequence - if not, it should behave like Stop
                    RhythmSlot::Stop | RhythmSlot::Continue => *next_event = None,
                    RhythmSlot::Rhythm(rhythm) => {
                        if let Some((sample_time, event)) =
                            rhythm.borrow_mut().next_until_time(sample_time)
                        {
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
            if let Some((rhythm_index, event_sample_time, event)) = next_due.clone() {
                if event_sample_time < sample_time {
                    *next_due = None; // consume
                    Some((rhythm_index, self.sample_offset + event_sample_time, event))
                }
                else {
                    None // not yet due
                }
            } else {
                None // no event available
            }
        } else {
            None
        }
    }
}

impl Iterator for Phrase {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, sample_time, event)) = self.next_event_until_time(SampleTime::MAX) {
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
    fn set_time_base(&mut self, time_base: BeatTimeBase) {
        for rhythm_slot in self.rhythm_slots.iter_mut() {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().set_time_base(time_base)
            }
        }
    }

    fn samples_per_step(&self) -> f64 {
        self.length.samples_per_step(&self.time_base)
    }
    fn pattern_length(&self) -> usize {
        // use our length's step, likely won't be used anyway for phrases
        self.length.steps() as usize
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)> {
        if let Some((_, sample_time, event)) = self.next_event_until_time(sample_time) {
            Some((sample_time, event))
        } else {
            None
        }
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.next_events.fill(None);
        // reset all rhythms in our slots as well
        for rhythm_slot in self.rhythm_slots.iter_mut() {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().reset()
            }
        }
    }
}
