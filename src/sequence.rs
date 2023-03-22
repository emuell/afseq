//! Combine multiple `Phrase` iterators into a single one to play them sequentially.

use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::{
    event::Event, phrase::RhythmIndex, time::SampleTimeDisplay, BeatTimeBase, Phrase, Rhythm,
    SampleOffset, SampleTime,
};

#[cfg(doc)]
use crate::EventIter;

// -------------------------------------------------------------------------------------------------

/// Sequencially arrange [`Phrase`] into a new [`EventIter`] to form simple arrangements.
///
/// The `run_until_time` function can be used to feed the entire sequence into a player engine.
#[derive(Clone, Debug)]
pub struct Sequence {
    time_base: BeatTimeBase,
    phrases: Rc<RefCell<Vec<Phrase>>>,
    phrase_index: usize,
    sample_position_in_phrase: SampleTime,
    sample_position: SampleTime,
    sample_offset: SampleOffset,
}

impl Sequence {
    /// Create a new sequence from a vector of [`Phrase`].
    pub fn new(time_base: BeatTimeBase, phrases: Vec<Phrase>) -> Self {
        let phrase_index = 0;
        let sample_position_in_phrase = 0;
        let sample_position = 0;
        let sample_offset = 0;
        Self {
            time_base,
            phrases: Rc::new(RefCell::new(phrases)),
            phrase_index,
            sample_position_in_phrase,
            sample_position,
            sample_offset,
        }
    }

    /// Read-only borrowed access to our phrases.
    pub fn phrases(&self) -> Ref<Vec<Phrase>> {
        self.phrases.borrow()
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn run_until_time<F>(&mut self, run_until_time: SampleTime, mut consumer: F)
    where
        F: FnMut(RhythmIndex, SampleTime, &Option<Event>),
    {
        debug_assert!(
            run_until_time >= self.sample_position,
            "can not rewind playback here"
        );
        while run_until_time - self.sample_position > 0 {
            let phrase_length_in_samples =
                self.current_phrase().length().to_samples(&self.time_base) as SampleTime;
            let next_phrase_start = phrase_length_in_samples - self.sample_position_in_phrase;
            let samples_to_run = run_until_time - self.sample_position;
            if next_phrase_start <= samples_to_run {
                // run current phrase until it ends
                let sample_position = self.sample_position;
                self.current_phrase_mut()
                    .run_until_time(sample_position + next_phrase_start, &mut consumer);
                // select next phrase in the sequence
                let mut last_phrase = self.current_phrase_mut().clone();
                self.phrase_index += 1;
                if self.phrase_index >= self.phrases().len() {
                    self.phrase_index = 0;
                }
                self.sample_position_in_phrase = 0;
                self.sample_position += next_phrase_start;
                // reset the new phrase or apply continues modes
                if self.phrases().len() > 1 {
                    let sample_offset = self.sample_position;
                    self.current_phrase_mut()
                        .reset_with_offset(sample_offset as i64, &mut last_phrase);
                }
            } else {
                // keep running the current phrase
                let sample_position = self.sample_position;
                self.current_phrase_mut()
                    .run_until_time(sample_position + samples_to_run, &mut consumer);
                self.sample_position_in_phrase += samples_to_run;
                self.sample_position += samples_to_run;
            }
        }
    }

    fn current_phrase(&self) -> Ref<Phrase> {
        let phrases: Ref<Vec<Phrase>> = self.phrases.borrow();
        Ref::map(phrases, |t| &t[self.phrase_index])
    }

    fn current_phrase_mut(&mut self) -> RefMut<Phrase> {
        let phrases = self.phrases.borrow_mut();
        RefMut::map(phrases, |t: &mut Vec<Phrase>| &mut t[self.phrase_index])
    }

    fn next_event(&mut self) -> Option<(SampleTime, Option<Event>)> {
        let event = self.current_phrase_mut().next();
        if let Some((sample_time, event)) = event {
            Some((
                (sample_time as i64 + self.sample_offset).max(0) as u64,
                event,
            ))
        } else {
            None
        }
    }
}

impl Iterator for Sequence {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_event()
    }
}

impl Rhythm for Sequence {
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
        self.sample_position = 0;
        self.sample_position_in_phrase = 0;
        // reset all our phrase iters
        let mut phrases = (*self.phrases).borrow_mut();
        for phrase in phrases.iter_mut() {
            phrase.reset()
        }
    }
}
