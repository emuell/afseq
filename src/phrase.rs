//! Stack multiple `Rhythm`S into a single one.

use std::{cell::RefCell, cmp::Ordering, fmt::Debug, rc::Rc};

use crate::{
    BeatTimeBase, BeatTimeStep, Event, InputParameter, InputParameterSet, Rhythm,
    RhythmEventTransform, RhythmIter, RhythmIterItem, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// A single slot in a [`Phrase`] vector.
#[derive(Clone, Debug, Default)]
pub enum RhythmSlot {
    /// Stop previous playing rhythm and/or simply play nothing. This can be useful to
    /// create empty placeholder slots in e.g. a [Sequence][`crate::Sequence`].
    #[default]
    Stop,
    /// Continue playing a previously played rhythm in a [Sequence][`crate::Sequence`].
    Continue,
    /// Play a shared rhytm in this slot. NB: This is a shared reference, in order to
    /// resolve 'Continue' modes in a [Sequence](`crate::Sequence`).
    Rhythm(Rc<RefCell<dyn Rhythm>>),
}

/// Convert an unboxed [`Rhythm`] to a [`RhythmSlot`]
impl<R> From<R> for RhythmSlot
where
    R: Rhythm + 'static,
{
    fn from(rhythm: R) -> RhythmSlot {
        RhythmSlot::Rhythm(Rc::new(RefCell::new(rhythm)))
    }
}

/// Convert a shared [`Rhythm`] to a [`RhythmSlot`]
impl From<Rc<RefCell<dyn Rhythm>>> for RhythmSlot {
    fn from(rhythm: Rc<RefCell<dyn Rhythm>>) -> RhythmSlot {
        RhythmSlot::Rhythm(rhythm)
    }
}

// -------------------------------------------------------------------------------------------------

/// Rhythm index in `PhraseIterItem`.
pub type RhythmIndex = usize;
/// Event as emitted by the Phrase, tagged with an additional rhythm index.
pub type PhraseIterItem = (RhythmIndex, RhythmIterItem);

// -------------------------------------------------------------------------------------------------

/// Combines multiple [`Rhythm`] into a new one, allowing to form more complex rhythms that are
/// meant to run together. Further it allows to run/evaluate rhythms until a specific sample time
/// is reached.
///
/// An example phrase is a drum-kit pattern where each instrument's pattern is defined separately
/// and then is combined into a single "big" pattern to play the entire kit together.
///
/// The `run_until_time` function is also used by [Sequence][`crate::Sequence`] to play a phrase
/// with a player engine.
#[derive(Clone)]
pub struct Phrase {
    time_base: BeatTimeBase,
    length: BeatTimeStep,
    input_parameters: InputParameterSet,
    rhythm_slots: Vec<RhythmSlot>,
    next_events: Vec<Option<PhraseIterItem>>,
    event_transform: Option<RhythmEventTransform>,
    sample_offset: SampleTime,
}

impl Debug for Phrase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericRhythm")
            .field("time_base", &self.time_base)
            .field("length", &self.length)
            .field("input_parameters", &self.input_parameters)
            .field("rhythm_slots", &self.rhythm_slots)
            // Skip event_transform, which has no Debug impl and next_events to reduce noise
            .field("sample_offset", &self.sample_offset)
            .finish()
    }
}

impl Phrase {
    /// Create a new phrase from a vector of [`RhythmSlot`] and the given length.
    /// NB: `RhythmSlot` has `Into` implementations, so you can also pass a vector of
    /// boxed or raw rhythm instance here.
    pub fn new<R: Into<RhythmSlot>>(
        time_base: BeatTimeBase,
        rhythm_slots: Vec<R>,
        length: BeatTimeStep,
    ) -> Self {
        let rhythm_slots = rhythm_slots
            .into_iter()
            .map(|r| r.into())
            .collect::<Vec<RhythmSlot>>();
        // collect input parameters from all slots
        let mut input_parameters = InputParameterSet::new();
        for slot in &rhythm_slots {
            if let RhythmSlot::Rhythm(rhythm) = slot {
                let rhythm = (**rhythm).borrow();
                for param in rhythm.input_parameters() {
                    // silently skip duplicate parameter ids
                    if !input_parameters
                        .iter()
                        .any(|p| p.borrow().id() == param.borrow().id())
                    {
                        input_parameters.push(Rc::clone(param));
                    }
                }
            }
        }
        let next_events = vec![None; rhythm_slots.len()];
        let event_transform = None;
        let sample_offset = 0;
        Self {
            time_base,
            length,
            input_parameters,
            rhythm_slots,
            next_events,
            event_transform,
            sample_offset,
        }
    }

    /// Read-only access to our phrase length.
    /// This is applied in [Sequence][`crate::Sequence`] only.
    pub fn length(&self) -> BeatTimeStep {
        self.length
    }

    /// Read-only access to our rhythm slots.
    pub fn rhythm_slots(&self) -> &[RhythmSlot] {
        &self.rhythm_slots
    }

    /// Mut access to our rhythm slots.
    pub fn rhythm_slots_mut(&mut self) -> &mut [RhythmSlot] {
        &mut self.rhythm_slots
    }

    /// Run rhythms until a given sample time is reached, calling the given `consumer`
    /// visitor function for all emitted events.
    pub fn consume_events_until_time<F>(&mut self, sample_time: SampleTime, consumer: &mut F)
    where
        F: FnMut(RhythmIndex, RhythmIterItem),
    {
        // emit and consume next events until we've reached the desired sample_time
        while let Some((rhythm_index, mut rhythm_item)) = self.next_event_until_time(sample_time) {
            debug_assert!(rhythm_item.time < sample_time);
            self.apply_event_transform(&mut rhythm_item);
            consumer(rhythm_index, rhythm_item);
        }
    }

    /// Move rhythms until a given sample time is reached, ignoring all events until that time.
    pub fn advance_until_time(&mut self, sample_time: SampleTime) {
        // skip next events in all rhythms
        for (rhythm_slot, next_event) in self
            .rhythm_slots
            .iter_mut()
            .zip(self.next_events.iter_mut())
        {
            // skip cached, next due events
            if let Some((_, event)) = next_event {
                if event.time >= sample_time {
                    // cached event is not yet due: no need to advance the slot
                    continue;
                }
                *next_event = None;
            }
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().advance_until_time(sample_time);
            }
        }
    }

    /// reset playback status and shift events to the given sample position.
    /// Further take over rhythms from the passed previously playing phrase for `RhythmSlot::Continue` slots.   
    pub fn reset_with_offset(&mut self, sample_offset: SampleTime, previous_phrase: &Phrase) {
        // reset rhythm iters, unless they are in continue mode. in continue mode, copy the slot
        // from the previously playing phrase and adjust sample offsets to fit.
        for (rhythm_index, rhythm_slot) in self.rhythm_slots.iter_mut().enumerate() {
            match rhythm_slot {
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
                    self.next_events[rhythm_index]
                        .clone_from(&previous_phrase.next_events[rhythm_index]);
                    // take over rhythm
                    rhythm_slot.clone_from(&previous_phrase.rhythm_slots[rhythm_index]);
                }
            }
        }
    }

    /// Apply custom event transform function, if any to all emitted events
    fn apply_event_transform(&self, rhythm_item: &mut RhythmIterItem) {
        if let Some(transform) = &self.event_transform {
            if let Some(event) = &mut rhythm_item.event {
                transform(event);
            }
        }
    }

    fn next_event_until_time(&mut self, sample_time: SampleTime) -> Option<PhraseIterItem> {
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
                        if let Some(event) = rhythm.borrow_mut().run_until_time(sample_time) {
                            *next_event = Some((rhythm_index, event));
                        } else {
                            *next_event = None;
                        }
                    }
                }
            }
        }
        // select the next from all pre-fetched events with the smallest sample time
        let next_due = self.next_events.iter_mut().reduce(|min, next| {
            if let Some((_, min_event)) = min {
                if let Some((_, next_event)) = next {
                    match min_event.time.cmp(&next_event.time) {
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
            if let Some((rhythm_index, event)) = next_due.clone() {
                if event.time < sample_time {
                    *next_due = None; // consume
                    Some((rhythm_index, event.with_offset(self.sample_offset)))
                } else {
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

/// Custom iterator impl for phrases:
/// returning a tuple of the rhythm index and the rhythm event.
impl Iterator for Phrase {
    type Item = PhraseIterItem;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_event_until_time(SampleTime::MAX)
    }
}

impl RhythmIter for Phrase {
    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset;
    }

    fn event_transform(&self) -> &Option<RhythmEventTransform> {
        &self.event_transform
    }

    fn set_event_transform(&mut self, transform: Option<RhythmEventTransform>) {
        self.event_transform = transform;
    }

    fn run_until_time(&mut self, sample_time: SampleTime) -> Option<RhythmIterItem> {
        self.next_event_until_time(sample_time)
            .map(|(_, event)| event)
    }

    fn advance_until_time(&mut self, sample_time: SampleTime) {
        self.advance_until_time(sample_time)
    }
}

impl Rhythm for Phrase {
    fn input_parameters(&self) -> &[Rc<RefCell<InputParameter>>] {
        &self.input_parameters
    }

    fn pattern_step_length(&self) -> f64 {
        // use our length's step, likely won't be used anyway for phrases
        self.length.samples_per_step(&self.time_base)
    }

    fn pattern_length(&self) -> usize {
        // use our length's step, likely won't be used anyway for phrases
        self.length.steps() as usize
    }

    fn time_base(&self) -> &BeatTimeBase {
        &self.time_base
    }

    fn set_time_base(&mut self, time_base: &BeatTimeBase) {
        self.time_base.clone_from(time_base);
        for rhythm_slot in &mut self.rhythm_slots {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().set_time_base(time_base);
            }
        }
    }

    fn set_trigger_event(&mut self, event: &Event) {
        for rhythm_slot in &mut self.rhythm_slots {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().set_trigger_event(event);
            }
        }
    }

    fn duplicate(&self) -> Rc<RefCell<dyn Rhythm>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.next_events.fill(None);
        // reset all rhythms in our slots as well
        for rhythm_slot in &mut self.rhythm_slots {
            if let RhythmSlot::Rhythm(rhythm) = rhythm_slot {
                rhythm.borrow_mut().reset();
            }
        }
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::prelude::*;

    fn create_phrase() -> Result<Phrase, String> {
        let beat_time = BeatTimeBase {
            samples_per_sec: 44100,
            beats_per_min: 130.0,
            beats_per_bar: 4,
        };

        let seed = 12312312312_u64;
        let kick_cycle = new_cycle_event_with_seed(
            "bd? [~ bd] ~ ~ bd [~ bd] _ ~ bd? [~ bd] ~ ~ bd [~ bd] [_ bd2] [~ bd _ ~]",
            seed,
        )?;
        let mut kick_pattern = beat_time.every_nth_beat(16.0).trigger(kick_cycle);
        kick_pattern.set_sample_offset(20); // test with offsets

        let snare_pattern = beat_time
            .every_nth_beat(2.0)
            .with_offset(BeatTimeStep::Beats(1.0))
            .with_event_transform(Rc::new(|event| {
                if let Event::NoteEvents(notes) = event {
                    for note in notes.iter_mut().flatten() {
                        note.note = Note::D4;
                    }
                }
            }))
            .trigger(new_note_event("C_5"));

        let hihat_pattern =
            beat_time
                .every_nth_sixteenth(2.0)
                .trigger(new_note_event("C_5").mutate({
                    let mut step = 0;
                    move |event| {
                        if let Event::NoteEvents(notes) = event {
                            for note in notes.iter_mut().flatten() {
                                note.volume = 1.0 / (step + 1) as f32;
                                step += 1;
                                if step >= 3 {
                                    step = 0;
                                }
                            }
                        }
                    }
                }));

        let hihat_pattern2 = beat_time
            .every_nth_sixteenth(2.0)
            .with_offset(BeatTimeStep::Sixteenth(1.0))
            .trigger(new_note_event("C_5").mutate({
                let mut vel_step = 0;
                let mut note_step = 0;
                move |event| {
                    if let Event::NoteEvents(notes) = event {
                        for note in notes.iter_mut().flatten() {
                            note.volume = 1.0 / (vel_step + 1) as f32 * 0.5;
                            vel_step += 1;
                            if vel_step >= 3 {
                                vel_step = 0;
                            }
                            note.note = Note::from((Note::C4 as u8) + 32 - note_step);
                            note_step += 1;
                            if note_step >= 32 {
                                note_step = 0;
                            }
                        }
                    }
                }
            }));

        let hihat_rhythm = Phrase::new(
            beat_time,
            vec![hihat_pattern, hihat_pattern2],
            BeatTimeStep::Bar(4.0),
        );

        let bass_notes = Scale::try_from((Note::C5, "aeolian")).unwrap().notes();
        let bass_pattern = beat_time
            .every_nth_eighth(1.0)
            .with_pattern([1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1].to_pattern())
            .trigger(new_note_event_sequence(vec![
                new_note((bass_notes[0], None, 0.5)),
                new_note((bass_notes[2], None, 0.5)),
                new_note((bass_notes[3], None, 0.5)),
                new_note((bass_notes[0], None, 0.5)),
                new_note((bass_notes[2], None, 0.5)),
                new_note((bass_notes[3], None, 0.5)),
                new_note((bass_notes[6].transposed(-12), None, 0.5)),
            ]));

        let synth_pattern =
            beat_time
                .every_nth_bar(4.0)
                .trigger(new_polyphonic_note_sequence_event(vec![
                    vec![
                        new_note(("C 4", None, 0.3)),
                        new_note(("D#4", None, 0.3)),
                        new_note(("G 4", None, 0.3)),
                    ],
                    vec![
                        new_note(("C 4", None, 0.3)),
                        new_note(("D#4", None, 0.3)),
                        new_note(("F 4", None, 0.3)),
                    ],
                    vec![
                        new_note(("C 4", None, 0.3)),
                        new_note(("D#4", None, 0.3)),
                        new_note(("G 4", None, 0.3)),
                    ],
                    vec![
                        new_note(("C 4", None, 0.3)),
                        new_note(("D#4", None, 0.3)),
                        new_note(("A#4", None, 0.3)),
                    ],
                ]));

        let fx_pattern =
            beat_time
                .every_nth_seconds(8.0)
                .trigger(new_polyphonic_note_sequence_event(vec![
                    vec![new_note(("C 4", None, 0.2)), None, None],
                    vec![None, new_note(("C 4", None, 0.2)), None],
                    vec![None, None, new_note(("F 4", None, 0.2))],
                ]));

        let tone_pattern = beat_time
            .every_nth_eighth(1.0)
            .trigger(new_cycle_event("[60 63 65 <58 ~>]/4")?);

        Ok(Phrase::new(
            beat_time,
            vec![
                RhythmSlot::from(kick_pattern),
                RhythmSlot::from(snare_pattern),
                RhythmSlot::from(hihat_rhythm),
                RhythmSlot::from(bass_pattern),
                RhythmSlot::from(fx_pattern),
                RhythmSlot::from(synth_pattern),
                RhythmSlot::from(tone_pattern),
            ],
            BeatTimeStep::Bar(8.0),
        ))
    }

    fn run_phrase(phrase: &mut Phrase, time: SampleTime) -> Vec<RhythmIterItem> {
        let mut events = Vec::new();
        while let Some(event) = phrase.run_until_time(time) {
            events.push(event)
        }
        events
    }

    // slow skip using run_until_time
    fn skip_phrase_by_running(phrase: &mut Phrase, time: SampleTime) {
        while phrase.run_until_time(time).is_some() {
            // ignore event
        }
    }

    // fast skip using skip_events_until_time
    fn skip_phrase_by_advancing(phrase: &mut Phrase, time: SampleTime) {
        phrase.advance_until_time(time)
    }

    #[test]
    fn skip_events() -> Result<(), String> {
        let sample_offset = 2345676;

        let mut phrase1 = create_phrase()?;
        phrase1.set_sample_offset(sample_offset);
        let mut events1 = Vec::new();

        let mut phrase2 = create_phrase()?;
        phrase2.set_sample_offset(sample_offset);
        let mut events2 = Vec::new();

        // run_time, advance_time
        let run_steps = [
            (1024, 1),
            (2000, 555432),
            (5012, 666),
            (200, 211),
            (100, 11200),
            (1024, 122),
            (8000, 5577432),
            (50700, 66),
            (21020, 2121),
            (1000, 100),
        ];

        let mut sample_time = sample_offset;
        for (run_time, seek_time) in run_steps {
            sample_time += run_time;
            events1.append(&mut run_phrase(&mut phrase1, sample_time));
            events2.append(&mut run_phrase(&mut phrase2, sample_time));

            sample_time += seek_time;
            skip_phrase_by_running(&mut phrase1, sample_time);
            skip_phrase_by_advancing(&mut phrase2, sample_time);
        }

        assert_eq!(events1, events2);

        Ok(())
    }
}
