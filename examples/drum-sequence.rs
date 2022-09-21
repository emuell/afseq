use std::rc::Rc;

use afseq::{
    emitter::BeatTimeBase, emitter_value_from_iter, value::fixed::FixedEmitterValue,
    value::mapped::MappedEmitterValue, EmitterEvent, EmitterSequence, InstrumentId, NoteEvent,
    SampleTime,
};

fn main() {
    let time_base = BeatTimeBase {
        beats_per_min: 120.0,
        beats_per_bar: 4,
        samples_per_sec: 44100,
    };

    const KICK: InstrumentId = 0;
    const SNARE: InstrumentId = 1;
    const HIHAT: InstrumentId = 2;

    let new_note = |instrument, note, velocity| {
        FixedEmitterValue::new(EmitterEvent::new_note(NoteEvent {
            instrument,
            note,
            velocity,
        }))
    };

    let new_mapped_note = |instrument: Option<usize>,
                           note: u32,
                           velocity: f32,
                           map: fn(&mut Vec<NoteEvent>) -> ()| {
        MappedEmitterValue::new(
            EmitterEvent::new_note(NoteEvent {
                instrument,
                note,
                velocity,
            }),
            move |event| {
                let mut event = event.clone();
                match &mut event {
                    EmitterEvent::NoteEvents(notes) => map(notes),
                    EmitterEvent::ParameterChangeEvent(_) => {}
                }
                event
            },
        )
    };

    let kick_pattern = time_base.every_nth_beat(4, new_note(Some(KICK), 60, 1.0));
    let snare_pattern = time_base.every_nth_beat_with_offset(8, 4, new_note(Some(SNARE), 60, 1.0));
    let hihat_pattern = time_base.every_nth_beat_with_offset(
        2,
        1,
        new_mapped_note(Some(HIHAT), 60, 1.0, |notes| {
            for note in notes {
                if note.velocity > 0.1 {
                    note.velocity -= 0.1;
                }
            }
        }),
    );

    let hihat_pattern2 = time_base.every_nth_beat(
        2,
        emitter_value_from_iter(
            new_note(Some(HIHAT), 60, 1.0)
                .map(|event| {
                    let mut mut_event = event;
                    match &mut mut_event {
                        EmitterEvent::NoteEvents(notes) => {
                            for note in notes {
                                note.velocity = 0.1;
                            }
                        }
                        EmitterEvent::ParameterChangeEvent(_) => {}
                    }
                    mut_event
                })
                .take(2),
        ),
    );

    let mut drum_sequence = EmitterSequence::new(vec![
        Rc::new(kick_pattern),
        Rc::new(snare_pattern),
        Rc::new(hihat_pattern),
        Rc::new(hihat_pattern2),
    ]);

    drum_sequence.run_until_time(
        (time_base.samples_per_bar() * 4.0) as SampleTime,
        |sample_time, event| {
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
        },
    )
}
