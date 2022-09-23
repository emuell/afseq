//! An experimental functional musical sequence generator.
//! Part of the [afplay](https://github.com/emuell/afplay) crates.

pub mod time;
pub use time::SampleTime;

pub mod event;
pub use event::{Event, EventIter};

pub mod rhythm;
pub use rhythm::Rhythm;

pub mod phrase;
pub use phrase::Phrase;

pub mod convert;

pub mod prelude;

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{Phrase, SampleTime};

    use super::{
        event::Event,
        event::NoteEvent,
        event::{fixed::*, mapped::*},
        rhythm::{beat_time::*, beat_time_sequence::*},
        time::{BeatTimeBase, BeatTimeStep},
    };

    #[test]
    fn test_sequencer() {
        let time_base = BeatTimeBase {
            beats_per_bar: 4,
            samples_per_sec: 44100,
            beats_per_min: 120.0,
        };

        let note_vector = FixedEventIter::new(Event::new_note_vector(vec![
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
        let note = MappedEventIter::new(
            Event::new_note(NoteEvent {
                instrument: None,
                note: 60,
                velocity: 1.0,
            }),
            |event| {
                let mut event = event;
                if let Event::NoteEvents(note_vector) = &mut event {
                    for note in note_vector {
                        note.note += 1;
                    }
                }
                event
            },
        );

        let beat_time_emitter =
            BeatTimeRhythm::new(time_base.clone(), BeatTimeStep::Beats(2.0), note);

        let beat_time_pattern_emitter = BeatTimeSequenceRhythm::new(
            time_base.clone(),
            BeatTimeStep::Beats(2.0),
            [1, 0, 0, 1],
            note_vector,
        );

        let mut phrase = Phrase::new(vec![
            Box::new(beat_time_emitter),
            Box::new(beat_time_pattern_emitter),
        ]);

        let mut num_note_events = 0;
        phrase.run_until_time((time_base.samples_per_beat() * 8.0) as SampleTime, {
            let num_note_events = &mut num_note_events;
            move |sample_time: SampleTime, event: &Option<Event>| {
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
        // 4 from beat, 2 from pattern
        assert_eq!(num_note_events, 4 + 2)
    }
}
