use std::sync::{Arc, RwLock};

use rust_music_theory::{note::Notes, scale};
use simplelog::*;

use afseq::prelude::*;

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    TermLogger::init(
        log::STATIC_MAX_LEVEL,
        ConfigBuilder::default().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap_or_else(|err| {
        log::error!("init_logger error: {:?}", err);
    });

    // preload samples
    let sample_pool = SamplePool::new();
    let KICK = sample_pool.load_sample("assets/kick.wav")?;
    let SNARE = sample_pool.load_sample("assets/snare.wav")?;
    let HIHAT = sample_pool.load_sample("assets/hihat.wav")?;
    let BASS = sample_pool.load_sample("assets/bass.wav")?;
    let SYNTH = sample_pool.load_sample("assets/synth.wav")?;
    // let TONE = sample_pool.load_sample("assets/tone.wav")?;
    let FX = sample_pool.load_sample("assets/fx.wav")?;

    // create event player
    let mut player = SamplePlayer::new(Arc::new(RwLock::new(sample_pool)), None)?;
    player.set_show_events(true);

    // define our time bases
    let second_time = SecondTimeBase {
        samples_per_sec: player.file_player().output_sample_rate(),
    };
    let beat_time = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: second_time.samples_per_sec,
    };

    // generate a few phrases
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

    // combine two hi hat rhythms into a phrase
    let hihat_rhythm = Phrase::new(
        beat_time,
        vec![hihat_pattern, hihat_pattern2],
        BeatTimeStep::Bar(4.0)
    );

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

    let fx_pattern =
        second_time
            .every_nth_seconds(8.0)
            .trigger(new_polyphonic_note_sequence_event(vec![
                vec![Some((FX, "C 3", 0.2)), None, None],
                vec![None, Some((FX, "C 4", 0.2)), None],
                vec![None, None, Some((FX, "F 4", 0.2))],
            ]));

    // arrange rhytms into phrases with equal row counts 
    let intro = Phrase::new(
        beat_time,
        vec![
            kick_pattern.into(),
            snare_pattern.into(),
            RhythmSlot::Stop,
            RhythmSlot::Stop,
            RhythmSlot::Stop,
            RhythmSlot::Stop,
        ],
        BeatTimeStep::Bar(4.0)
    );
    let intro_with_bass = Phrase::new(
        beat_time,
        vec![
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            hihat_rhythm.into(),
            bass_pattern.into(),
            RhythmSlot::Stop,
            RhythmSlot::Stop,
        ],
        BeatTimeStep::Bar(4.0)
    );
    let main = Phrase::new(
        beat_time,
        vec![
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            synth_pattern.into(),
            RhythmSlot::Stop,
        ],
        BeatTimeStep::Bar(32.0)
    );

    let main_with_fx = Phrase::new(
        beat_time,
        vec![
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            RhythmSlot::Continue,
            fx_pattern.into(),
        ],
        BeatTimeStep::Bar(32.0)
    );

    // form a sequence from phrases
    let mut sequence = Sequence::new(
        beat_time,
        vec![
            intro,
            intro_with_bass,
            main,
            main_with_fx,
        ],
    );

    // play the sequence and dump events to stdout
    const RESET_PLAYBACK_POS: bool = true;
    player.run(&mut sequence, &beat_time, RESET_PLAYBACK_POS);

    Ok(())
}
