//! Combine multiple `Rythm` iterators into a single one to play them at the same time.

use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    fmt::Debug,
    rc::Rc,
};

use crate::{
    event::Event, prelude::BeatTimeStep, time::SampleTimeDisplay, BeatTimeBase, Rhythm,
    SampleOffset, SampleTime,
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
#[derive(Debug)]
pub enum RhythmSlot {
    /// Stop previous playing rhythm and/or simply play nothing.
    /// This can be useful to create empty placeholder slots in e.g. a [`Sequence`].
    Stop,
    /// Continue playing a previously played rhythm in a [`Sequence`].
    Continue,
    /// Play the given boxed rhytm in this slot.
    Rhythm(Box<dyn Rhythm>),
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
#[derive(Clone, Debug)]
pub struct Phrase {
    time_base: BeatTimeBase,
    offset: BeatTimeStep,
    sample_offset: SampleOffset,
    length: BeatTimeStep,
    rhythm_slots: Rc<RefCell<Vec<RhythmSlot>>>,
    next_events: Vec<Option<RhythmEvent>>,
    held_back_event: Option<RhythmEvent>,
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
    /// This is applied in [`Sequence`] only.
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

    /// reset playback status and shift offset to the given sample position.
    /// Further take over rhythms from the passed previously playing phrase for RhythmSlot::Continue slots.   
    pub fn reset_with_offset(&mut self, sample_offset: SampleOffset, previous_phrase: &mut Phrase) {
        // reset rhythm iters, unless they are in continue mode. in contine mode, copy the slot
        // from the previously playing phrase and adjust sample offsets to fit.
        let mut previous_rhythms = previous_phrase.rhythm_slots.borrow_mut();
        let mut current_rhythms = self.rhythm_slots.borrow_mut();
        for rhythm_index in 0..current_rhythms.len() {
            match &mut current_rhythms[rhythm_index] {
                RhythmSlot::Rhythm(rhythm) => {
                    rhythm.reset();
                    rhythm.set_sample_offset(sample_offset);
                    self.next_events[rhythm_index] = None;
                    if let Some((index, _, _)) = self.held_back_event {
                        if index == rhythm_index {
                            self.held_back_event = None
                        }
                    }
                }
                RhythmSlot::Stop => {
                    self.next_events[rhythm_index] = None;
                    if let Some((index, _, _)) = self.held_back_event {
                        if index == rhythm_index {
                            self.held_back_event = None
                        }
                    }
                }
                RhythmSlot::Continue => {
                    // take over pending events
                    self.next_events[rhythm_index] =
                        std::mem::replace(&mut previous_phrase.next_events[rhythm_index], None);
                    if let Some((index, _, _)) = previous_phrase.held_back_event {
                        if index == rhythm_index {
                            self.held_back_event =
                                std::mem::replace(&mut previous_phrase.held_back_event, None)
                        }
                    }
                    // take over rhythm
                    current_rhythms[rhythm_index] = std::mem::replace(
                        &mut previous_rhythms[rhythm_index],
                        RhythmSlot::Continue,
                    );
                }
            }
        }
    }

    fn next_event(&mut self) -> Option<RhythmEvent> {
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
                    // NB: Continue mode is resolved by the Sequence - if not, it should behave like Stop
                    RhythmSlot::Stop | RhythmSlot::Continue => *next_event = None,
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
