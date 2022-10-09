use std::{cell::RefCell, rc::Rc};

use rust_music_theory::{note::Notes, scale};

use afseq::prelude::*;

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create sample pool
    let sample_pool = Rc::new(RefCell::new(SamplePool::new()));

    // preload samples
    let KICK = sample_pool.borrow_mut().load_sample("assets/kick.wav")?;
    let SNARE = sample_pool.borrow_mut().load_sample("assets/snare.wav")?;
    let HIHAT = sample_pool.borrow_mut().load_sample("assets/hat.wav")?;
    let BASS = sample_pool.borrow_mut().load_sample("assets/bass.wav")?;
    let SYNTH = sample_pool.borrow_mut().load_sample("assets/synth.wav")?;
    let _TONE = sample_pool.borrow_mut().load_sample("assets/tone.wav")?;
    let FX = sample_pool.borrow_mut().load_sample("assets/fx.wav")?;

    // create event player
    let mut player = SamplePlayer::new(sample_pool)?;
    player.set_show_events(true);

    // define our time bases
    let second_time = SecondTimeBase {
        samples_per_sec: player.sample_rate(),
    };
    let beat_time = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: player.sample_rate(),
    };

    // generate a simple phrase
    let kick_pattern = beat_time
        .every_nth_sixteenth(1.0)
        .with_pattern([
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 1, 0, 0, //
        ])
        .trigger(new_note_event(KICK, "C_4", 1.0));

    let snare_pattern = beat_time
        .every_nth_beat(2.0)
        .with_offset(BeatTimeStep::Beats(1.0))
        .trigger(new_note_event(SNARE, "C_4", 1.0));

    let hihat_pattern =
        beat_time
            .every_nth_sixteenth(2.0)
            .trigger(new_note_event(HIHAT, "C_4", 1.0).mutate({
                let mut step = 0;
                move |mut event| {
                    if let Event::NoteEvents(notes) = &mut event {
                        for (_index, note) in notes.iter_mut().enumerate() {
                            if let Some(note) = note {
                                note.velocity = 1.0 / (step + 1) as f32;
                                step += 1;
                                if step >= 3 {
                                    step = 0;
                                }
                            }
                        }
                    }
                    event
                }
            }));
    let hihat_pattern2 = beat_time
        .every_nth_sixteenth(2.0)
        .with_offset(BeatTimeStep::Sixteenth(1.0))
        .trigger(new_note_event(HIHAT, "C_4", 1.0).mutate({
            let mut vel_step = 0;
            let mut note_step = 0;
            move |mut event| {
                if let Event::NoteEvents(notes) = &mut event {
                    for (_index, note) in notes.iter_mut().enumerate() {
                        if let Some(note) = note {
                            note.velocity = 1.0 / (vel_step + 1) as f32 * 0.5;
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
                event
            }
        }));

    let hihat_rhythm = Phrase::new(
        beat_time,
        vec![Box::new(hihat_pattern), Box::new(hihat_pattern2)],
    )
    .with_offset(BeatTimeStep::Bar(4.0));

    let bass_notes = scale::Scale::from_regex("c aeolian")?.notes();
    let bass_pattern = beat_time
        .every_nth_eighth(1.0)
        .with_pattern([1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1])
        .trigger(new_note_event_sequence(vec![
            (BASS, Note::from(&bass_notes[0]), 0.5),
            (BASS, Note::from(&bass_notes[2]), 0.5),
            (BASS, Note::from(&bass_notes[3]), 0.5),
            (BASS, Note::from(&bass_notes[0]), 0.5),
            (BASS, Note::from(&bass_notes[2]), 0.5),
            (BASS, Note::from(&bass_notes[3]), 0.5),
            (BASS, Note::from(&bass_notes[6]) - 12, 0.5),
        ]));

    let synth_pattern = beat_time
        .every_nth_bar(4.0)
        .with_offset(BeatTimeStep::Bar(8.0))
        .trigger(new_polyphonic_note_sequence_event(vec![
            vec![
                (SYNTH, "C 3", 0.3),
                (SYNTH, "D#3", 0.3),
                (SYNTH, "G 3", 0.3),
            ],
            vec![
                (SYNTH, "C 3", 0.3),
                (SYNTH, "D#3", 0.3),
                (SYNTH, "F 3", 0.3),
            ],
            vec![
                (SYNTH, "C 3", 0.3),
                (SYNTH, "D#3", 0.3),
                (SYNTH, "G 3", 0.3),
            ],
            vec![
                (SYNTH, "C 3", 0.3),
                (SYNTH, "D#3", 0.3),
                (SYNTH, "A#3", 0.3),
            ],
        ]));

    let fx_pattern = second_time
        .every_nth_seconds(8.0)
        .with_offset(48.0)
        .trigger(new_note_event_sequence(vec![
            (FX, "C 3", 0.1),
            (FX, "C 4", 0.1),
            (FX, "F 4", 0.1),
        ]));

    let mut phrase = Phrase::new(
        beat_time,
        vec![
            Box::new(kick_pattern),
            Box::new(snare_pattern),
            Box::new(hihat_rhythm),
            Box::new(bass_pattern),
            Box::new(synth_pattern),
            Box::new(fx_pattern),
        ],
    );

    // play the phrase and dump events to stdout
    player.run(&mut phrase, &beat_time);

    Ok(())
}
