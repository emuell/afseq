//! Combine multiple `Phrase` iterators into a single one, in order to play them sequentially.

use crate::{
    event::Event, phrase::RhythmIndex, time::SampleTimeDisplay, BeatTimeBase, Phrase, Rhythm,
    SampleTime,
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
    phrases: Vec<Phrase>,
    phrase_index: usize,
    sample_position_in_phrase: SampleTime,
    sample_position: SampleTime,
    sample_offset: SampleTime,
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
            phrases,
            phrase_index,
            sample_position_in_phrase,
            sample_position,
            sample_offset,
        }
    }

    /// returns maximum rhythm count in all phrases.
    pub fn rhythm_slot_count(&self) -> usize {
        let mut count = 0;
        for phrase in self.phrases.iter() {
            count = count.max(phrase.rhythm_slots().len());
        }
        count
    }

    /// Read-only borrowed access to our phrases.
    pub fn phrases(&self) -> &Vec<Phrase> {
        &self.phrases
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn run_until_time<F>(&mut self, run_until_time: SampleTime, consumer: &mut F)
    where
        F: FnMut(RhythmIndex, SampleTime, Option<Event>),
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
                    .run_until_time(sample_position + next_phrase_start, consumer);
                // select next phrase in the sequence
                let previous_phrase = self.current_phrase_mut().clone();
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
                        .reset_with_offset(sample_offset, &previous_phrase);
                }
            } else {
                // keep running the current phrase
                let sample_position = self.sample_position;
                self.current_phrase_mut()
                    .run_until_time(sample_position + samples_to_run, consumer);
                self.sample_position_in_phrase += samples_to_run;
                self.sample_position += samples_to_run;
            }
        }
    }

    fn current_phrase(&self) -> &Phrase {
        &self.phrases[self.phrase_index]
    }

    fn current_phrase_mut(&mut self) -> &mut Phrase {
        &mut self.phrases[self.phrase_index]
    }

    fn next_event_until_time(
        &mut self,
        sample_time: SampleTime,
    ) -> Option<(SampleTime, Option<Event>)> {
        let event = self.current_phrase_mut().next_until_time(sample_time);
        if let Some((sample_time, event)) = event {
            Some((sample_time + self.sample_offset, event))
        } else {
            None
        }
    }
}

impl Iterator for Sequence {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_event_until_time(SampleTime::MAX)
    }
}

impl Rhythm for Sequence {
    fn time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }
    fn update_time_base(&mut self, time_base: &BeatTimeBase) {
        for phrase in self.phrases.iter_mut() {
            phrase.update_time_base(time_base);
        }
    }

    fn samples_per_step(&self) -> f64 {
        self.current_phrase().samples_per_step()
    }
    fn pattern_length(&self) -> usize {
        self.current_phrase().pattern_length()
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)> {
        self.next_event_until_time(sample_time)
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset our own iter state
        self.sample_position = 0;
        self.sample_position_in_phrase = 0;
        // reset all our phrase iters
        for phrase in self.phrases.iter_mut() {
            phrase.reset()
        }
    }
}

/// Allow sending sequences accross threads: We only do so, to allow building sequences outside
/// the thread they are played in, and never reuse phrases from other already running sequences.
///
/// So this is **safe as long as new sequences only reference phrases within itself**.
unsafe impl Send for Sequence {}
