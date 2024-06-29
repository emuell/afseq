//! Arrange multiple `Phrase`S into a single `Rhythm`.

use crate::{event::Event, phrase::RhythmIndex, BeatTimeBase, Phrase, Rhythm, SampleTime};

#[cfg(doc)]
use crate::EventIter;

// -------------------------------------------------------------------------------------------------

/// Sequentially arrange [`Phrase`] into a new [`EventIter`] to form simple arrangements.
///
/// The `consume_events_until_time` function can be used to feed the entire sequence into a
/// player engine.
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

    /// Read-only borrowed access to our time base.
    pub fn time_base(&self) -> &BeatTimeBase {
        &self.time_base
    }

    /// Read-only borrowed access to our phrases.
    pub fn phrases(&self) -> &Vec<Phrase> {
        &self.phrases
    }

    /// returns maximum rhythm count in all phrases.
    pub fn phrase_rhythm_slot_count(&self) -> usize {
        let mut count = 0;
        for phrase in &self.phrases {
            count = count.max(phrase.rhythm_slots().len());
        }
        count
    }

    /// Run rhythms until a given sample time is reached, calling the given `visitor`
    /// function for all emitted events to consume them.
    pub fn consume_events_until_time<F>(&mut self, run_until_time: SampleTime, consumer: &mut F)
    where
        F: FnMut(RhythmIndex, SampleTime, Option<Event>, SampleTime),
    {
        debug_assert!(
            run_until_time >= self.sample_position,
            "can not rewind playback here"
        );
        while run_until_time - self.sample_position > 0 {
            let (next_phrase_start, samples_to_run) =
                self.samples_until_next_phrase(run_until_time);
            if next_phrase_start <= samples_to_run {
                // run current phrase until it ends
                let sample_position = self.sample_position;
                self.current_phrase_mut()
                    .consume_events_until_time(sample_position + next_phrase_start, consumer);
                // select next phrase in the sequence
                let previous_phrase = self.current_phrase_mut().clone();
                self.phrase_index = (self.phrase_index + 1) % self.phrases().len();
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
                    .consume_events_until_time(sample_position + samples_to_run, consumer);
                self.sample_position_in_phrase += samples_to_run;
                self.sample_position += samples_to_run;
            }
        }
    }

    /// Seek sequence until a given sample time is reached, ignoring all events.
    pub fn skip_events_until_time(&mut self, run_until_time: SampleTime) {
        debug_assert!(
            run_until_time >= self.sample_position,
            "can not rewind playback here"
        );
        while run_until_time - self.sample_position > 0 {
            let (next_phrase_start, samples_to_run) =
                self.samples_until_next_phrase(run_until_time);
            if next_phrase_start <= samples_to_run {
                // run current phrase until it ends
                let sample_position = self.sample_position;
                self.current_phrase_mut()
                    .skip_events_until_time(sample_position + next_phrase_start);
                // select next phrase in the sequence
                let previous_phrase = self.current_phrase_mut().clone();
                self.phrase_index = (self.phrase_index + 1) % self.phrases().len();
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
                    .skip_events_until_time(sample_position + samples_to_run);
                self.sample_position_in_phrase += samples_to_run;
                self.sample_position += samples_to_run;
            }
        }
    }

    /// Reset all rhythms in our phrases to their initial state.
    pub fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset our own iter state
        self.sample_position = 0;
        self.sample_position_in_phrase = 0;
        // reset all our phrase iters
        for phrase in &mut self.phrases {
            phrase.reset();
        }
    }

    fn current_phrase(&self) -> &Phrase {
        &self.phrases[self.phrase_index]
    }

    fn current_phrase_mut(&mut self) -> &mut Phrase {
        &mut self.phrases[self.phrase_index]
    }

    fn samples_until_next_phrase(&self, run_until_time: u64) -> (u64, u64) {
        let phrase_length_in_samples =
            self.current_phrase().length().to_samples(&self.time_base) as SampleTime;
        let next_phrase_start = phrase_length_in_samples - self.sample_position_in_phrase;
        let samples_to_run = run_until_time - self.sample_position;
        (next_phrase_start, samples_to_run)
    }
}
