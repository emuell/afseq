use std::{
    path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use anyhow::anyhow;
use simplelog::*;

use afseq::prelude::*;

// -------------------------------------------------------------------------------------------------

#[cfg(feature = "dhat-profiler")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// -------------------------------------------------------------------------------------------------

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-profiler")]
    let profiler = dhat::Profiler::builder().trim_backtraces(Some(100)).build();

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
    fn sample_path(file_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(
            path::PathBuf::from(format!("./examples/assets/{file_name}"))
                .to_str()
                .ok_or(anyhow!(
                    "Failed to create asset path for file '{}'",
                    file_name
                ))?
                .to_string(),
        )
    }
    let sample_pool = SamplePool::new();
    let KICK = sample_pool.load_sample(&sample_path("kick.wav")?)?;
    let SNARE = sample_pool.load_sample(&sample_path("snare.wav")?)?;
    let HIHAT = sample_pool.load_sample(&sample_path("hihat.wav")?)?;
    let BASS = sample_pool.load_sample(&sample_path("bass.wav")?)?;
    let SYNTH = sample_pool.load_sample(&sample_path("synth.wav")?)?;
    // let TONE = sample_pool.load_sample(&sample_path("tone.wav")?)?;
    let FX = sample_pool.load_sample(&sample_path("fx.wav")?)?;

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
    let cycle =
        new_cycle_event("bd [~ bd] ~ ~ bd [~ bd] _ ~ bd [~ bd] ~ ~ bd [~ bd] [_ bd2] [~ bd _ ~]")?
            .with_mappings(&[
                ("bd", vec![new_note("c4")]),
                ("bd2", vec![new_note(("c4", None, 0.5))]),
            ]);

    let kick_pattern = beat_time
        .every_nth_beat(16.0)
        .with_instrument(KICK)
        .trigger(cycle);

    let snare_pattern = beat_time
        .every_nth_beat(2.0)
        .with_offset(BeatTimeStep::Beats(1.0))
        .with_instrument(SNARE)
        .trigger(new_note_event("C_5"));

    let hihat_pattern = beat_time
        .every_nth_sixteenth(2.0)
        .with_instrument(HIHAT)
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
        .with_instrument(HIHAT)
        .with_pattern([1.0, 0.5].to_pattern())
        .with_gate(ProbabilityGate::new(None))
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

    // combine two hi hat rhythms into a phrase
    let hihat_rhythm = Phrase::new(
        beat_time,
        vec![hihat_pattern, hihat_pattern2],
        BeatTimeStep::Bar(4.0),
    );

    let bass_notes = Scale::try_from((Note::C5, "aeolian"))?.notes();
    let bass_pattern = beat_time
        .every_nth_eighth(1.0)
        .with_instrument(BASS)
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

    let synth_pattern = beat_time.every_nth_bar(4.0).with_instrument(SYNTH).trigger(
        new_polyphonic_note_sequence_event(vec![
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
        ]),
    );

    let fx_pattern = beat_time
        .every_nth_seconds(8.0)
        .with_instrument(FX)
        .trigger(new_polyphonic_note_sequence_event(vec![
            vec![new_note(("C 4", None, 0.2)), None, None],
            vec![None, new_note(("C 4", None, 0.2)), None],
            vec![None, None, new_note(("F 4", None, 0.2))],
        ]));

    // arrange rhytms into phrases and sequence up these phrases to create a litte arrangement
    let mut sequence = Sequence::new(
        beat_time,
        vec![
            Phrase::new(
                beat_time,
                vec![
                    RhythmSlot::from(kick_pattern),
                    RhythmSlot::from(snare_pattern),
                    RhythmSlot::Stop, // hihat
                    RhythmSlot::Stop, // bass
                    RhythmSlot::Stop, // synth
                    RhythmSlot::Stop, // fx
                ],
                BeatTimeStep::Bar(8.0),
            ),
            Phrase::new(
                beat_time,
                vec![
                    RhythmSlot::Continue, // kick
                    RhythmSlot::Continue, // snare
                    RhythmSlot::from(hihat_rhythm),
                    RhythmSlot::from(bass_pattern),
                    RhythmSlot::Stop, // synth
                    RhythmSlot::Stop, // fx
                ],
                BeatTimeStep::Bar(8.0),
            ),
            Phrase::new(
                beat_time,
                vec![
                    RhythmSlot::Continue, // kick
                    RhythmSlot::Continue, // snare
                    RhythmSlot::Continue, // hihat
                    RhythmSlot::Continue, // bass
                    RhythmSlot::from(synth_pattern),
                    RhythmSlot::Stop, // fx
                ],
                BeatTimeStep::Bar(16.0),
            ),
            Phrase::new(
                beat_time,
                vec![
                    RhythmSlot::Continue, // kick
                    RhythmSlot::Continue, // snare
                    RhythmSlot::Continue, // hihat
                    RhythmSlot::Continue, // bass
                    RhythmSlot::Continue, // synth
                    RhythmSlot::from(fx_pattern),
                ],
                BeatTimeStep::Bar(16.0),
            ),
        ],
    );

    // stop on Control-C
    let stop_running = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler({
        let stop_running = stop_running.clone();
        move || {
            stop_running.store(true, Ordering::Relaxed);
        }
    })?;

    // play the sequence and dump events to stdout
    let reset_playback_pos = false;
    player.run_until(&mut sequence, &beat_time, reset_playback_pos, || {
        stop_running.load(Ordering::Relaxed)
    });

    #[cfg(feature = "dhat-profiler")]
    drop(profiler);

    Ok(())
}
