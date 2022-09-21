use std::cell::RefCell;

use afseq::{
    events::{
        fixed::ToMappedNotesEmitterValue, new_note, InstrumentId, ToFixedPatternEventValue,
        ToPatternEventValueSequence,
    },
    time::BeatTimeBase,
    Phrase, SampleTime,
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
    const BASS: InstrumentId = 3;

    let kick_pattern = time_base.every_nth_beat(4, new_note(Some(KICK), 60, 1.0).to_event());
    let snare_pattern =
        time_base.every_nth_beat_with_offset(8, 4, new_note(Some(SNARE), 60, 1.0).to_event());
    let hihat_pattern = time_base.every_nth_beat_with_offset(
        2,
        1,
        new_note(Some(HIHAT), 60, 1.0).to_event().map_notes(|note| {
            let mut note = note;
            if note.velocity > 0.1 {
                note.velocity -= 0.1;
            }
            note
        }),
    );

    let bass_note_sequence = vec![
        new_note(Some(BASS), 60, 1.0),
        new_note(Some(BASS), 64, 0.5),
        new_note(Some(BASS), 48, 0.70),
    ]
    .to_event_sequence();

    let bass_notes_pattern =
        time_base.every_nth_beat_with_pattern(1, vec![true, false, true, true], bass_note_sequence);

    let mut phrase = Phrase::new(vec![
        Box::new(RefCell::new(kick_pattern)),
        Box::new(RefCell::new(snare_pattern)),
        Box::new(RefCell::new(hihat_pattern)),
        Box::new(RefCell::new(bass_notes_pattern)),
    ]);

    phrase.run_until_time(
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
