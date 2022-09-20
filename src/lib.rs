use std::rc::Rc;

pub mod emitter;
pub mod value;

mod event;
pub use event::*;

// -------------------------------------------------------------------------------------------------

/// Sample time value type emitted by the [`Emitter`] trait.
type SampleTime = usize;

// -------------------------------------------------------------------------------------------------

/// `EmitterValue` is a value which is emitted by the [`Emitter`] trait iter. EmitterValues are
/// iters by themselves, so they may also change over time.
///
/// The most simple example of an `EmitterValue` is a constant note or DSP parameter value change.
/// Being an iterator, this value also then can change over time to e.g. produce note sequences
/// in an arpeggiator or parameter automation.
pub trait EmitterValue: Iterator<Item = EmitterEvent> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// An `Emittor` is an iterator which emits optional `EmitterValue`] at a given sample time.
///
/// An Emittor is what triggers events rythmically or periodically in a sequencer, and produces
/// values that happen at a given sample time. Players will use the sample time to schedule the
/// events in the audio stream. When a defined beat-based time-base is used in emitter impls,
/// such emitters may produce beat based values internally too, but those beat times always will
/// render down to sample times.
pub trait Emitter: Iterator<Item = (SampleTime, Option<EmitterEvent>)> {
    /// Access to the emitters actual value
    fn current_value(&self) -> &dyn EmitterValue;
    /// Access to the emitters last emitted time
    fn current_sample_time(&self) -> SampleTime;

    /// Resets/rewinds the emitter to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

/// An `EmitterSequence` combines and runs one or more [`Emitter`] at the same time, allowing to
/// form more complex emitters that should be run together.
///
/// An example sequence would be a drum kit pattern where each instrument's pattern is defined
/// separately and then combined into a single one.
pub struct EmitterSequence {
    emitters: Vec<Rc<dyn Emitter>>,
}

impl EmitterSequence {
    /// Run all emitters in the sequence until a given sample time is reached and call given
    /// visitor function for all emitted events.
    pub fn run_until_time<F>(&mut self, run_sample_time: SampleTime, mut visitor: F)
    where
        F: FnMut(SampleTime, &Option<EmitterEvent>),
    {
        for emitter in self.emitters.iter_mut() {
            if emitter.current_sample_time() < run_sample_time {
                let mut_emitter = Rc::get_mut(emitter).unwrap();
                for (sample_time, event) in mut_emitter {
                    visitor(sample_time, &event);
                    if sample_time >= run_sample_time {
                        break;
                    }
                }
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{EmitterSequence, SampleTime};
    use std::rc::Rc;

    use super::{
        emitter::{beat_time::*, beat_time_pattern::*, BeatTimeBase, BeatTimeStep},
        value::{fixed::*, mapped::*},
        EmitterEvent, NoteEvent,
    };

    #[test]
    fn test_sequencer() {
        let time_base = BeatTimeBase {
            beats_per_bar: 4,
            samples_per_sec: 44100,
            beats_per_min: 120.0,
        };

        let note_vector = FixedEmitterValue::new(EmitterEvent::new_note_vector(vec![
            NoteEvent {
                instrument: None,
                note: 60,
                velocity: 1.0,
            },
            NoteEvent {
                instrument: None,
                note: 64,
                velocity: 1.0,
            },
            NoteEvent {
                instrument: None,
                note: 68,
                velocity: 1.0,
            },
        ]));
        let note = MappedEmitterValue::new(
            EmitterEvent::new_note(NoteEvent {
                instrument: None,
                note: 60,
                velocity: 1.0,
            }),
            |event| {
                let mut new_event = event.clone();
                if let EmitterEvent::NoteEvents(note_vector) = &mut new_event {
                    for note in note_vector {
                        note.note += 1;
                    }
                }
                new_event
            },
        );

        let beat_time_emitter =
            BeatTimeEmitter::new(time_base.clone(), BeatTimeStep::Beats(2), note);

        let beat_time_pattern_emitter = BeatTimePatternEmitter::new(
            time_base.clone(),
            BeatTimeStep::Beats(2),
            vec![true, false, false, true],
            note_vector,
        );

        let mut sequence = EmitterSequence {
            emitters: vec![
                Rc::new(beat_time_emitter),
                Rc::new(beat_time_pattern_emitter),
            ],
        };

        let mut num_note_events = 0;
        sequence.run_until_time((time_base.samples_per_beat() * 8.0) as SampleTime, {
            let num_note_events = &mut num_note_events;
            move |sample_time: SampleTime, event: &Option<EmitterEvent>| {
                if event.is_some() {
                    *num_note_events += 1;
                }
                println!(
                    "{:.1} ({:08}) -> {}",
                    sample_time as f64 / time_base.samples_per_beat(),
                    sample_time,
                    match event {
                        Some(event) => {
                            format!("{:?}", event)
                        }
                        None => "---".to_string(),
                    }
                );
            }
        });
        // 5 from beat, 3 from pattern
        assert_eq!(num_note_events, 5 + 3)
    }
}
