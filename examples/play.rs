use std::collections::HashMap;

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlayer,
    AudioOutput, DefaultAudioOutput, FilePlaybackOptions,
};

use afseq::prelude::*;

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let mut player = AudioFilePlayer::new(audio_output.sink(), None);

    // preload all samples
    let KICK: InstrumentId = unique_instrument_id();
    let SNARE: InstrumentId = unique_instrument_id();
    let HIHAT: InstrumentId = unique_instrument_id();
    let BASS: InstrumentId = unique_instrument_id();
    let SYNTH: InstrumentId = unique_instrument_id();
    let FX: InstrumentId = unique_instrument_id();

    let load_file = |file_name| {
        PreloadedFileSource::new(file_name, None, FilePlaybackOptions::default()).unwrap()
    };

    let sample_pool: HashMap<InstrumentId, PreloadedFileSource> = HashMap::from([
        (KICK, load_file("assets/kick.wav")),
        (SNARE, load_file("assets/snare.wav")),
        (HIHAT, load_file("assets/hat.wav")),
        (BASS, load_file("assets/bass.wav")),
        (SYNTH, load_file("assets/synth.wav")),
        (FX, load_file("assets/fx.wav")),
    ]);

    // define our time bases
    let second_time_base = SecondTimeBase {
        samples_per_sec: player.output_sample_rate(),
    };
    let beat_time_base = BeatTimeBase {
        beats_per_min: 130.0,
        beats_per_bar: 4,
        samples_per_sec: player.output_sample_rate(),
    };

    // generate a simple sequence
    let kick_pattern = beat_time_base
        .every_nth_sixteenth(1.0)
        .with_pattern([
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 0, 0, //
            1, 0, 0, 0, /**/ 0, 0, 1, 0, /**/ 0, 0, 1, 0, /**/ 0, 1, 0, 0, //
        ])
        .trigger(new_note_event(KICK, "C_4", 1.0));

    let snare_pattern = beat_time_base
        .every_nth_beat(2.0)
        .with_offset(BeatTimeStep::Beats(1.0))
        .trigger(new_note_event(SNARE, "C_4", 1.0));

    let hihat_pattern = beat_time_base.every_nth_sixteenth(2.0).trigger(
        new_note_event(HIHAT, "C_4", 1.0).map_notes({
            let mut step = 0;
            move |mut note, _voice_index| {
                note.velocity = 1.0 / (step + 1) as f32;
                step += 1;
                if step >= 3 {
                    step = 0;
                }
                note
            }
        }),
    );
    let hihat_pattern2 = beat_time_base
        .every_nth_sixteenth(2.0)
        .with_offset(BeatTimeStep::Sixteenth(1.0))
        .trigger(new_note_event(HIHAT, "C_4", 1.0).map_notes({
            let mut vel_step = 0;
            let mut note_step = 0;
            move |mut note, _voice_index| {
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
                note
            }
        }));

    let hihat_rhythm = Phrase::new(
        beat_time_base,
        vec![Box::new(hihat_pattern), Box::new(hihat_pattern2)],
    )
    .with_offset(BeatTimeStep::Bar(4.0));

    let bass_pattern = beat_time_base
        .every_nth_eighth(1.0)
        .with_pattern([
            1, 0, 0, 0, /**/ 1, 0, 1, 0, /**/ 0, 0, 0, 1, /**/ 0, 0, 0, 0, /**/
            1, 0, 0, 0, /**/ 1, 0, 1, 0, /**/ 0, 0, 0, 0, /**/ 1, 0, 0, 0, /**/
        ])
        .trigger(new_note_event_sequence(vec![
            (BASS, "C 4", 0.5),
            (BASS, "C 4", 0.5),
            (BASS, "F 4", 0.5),
            (BASS, "A#3", 0.5),
        ]));

    let synth_pattern = beat_time_base
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

    let fx_pattern = second_time_base
        .every_nth_seconds(8.0)
        .with_offset(48.0)
        .trigger(new_note_event_sequence(vec![
            (FX, "C 3", 0.1),
            (FX, "C 4", 0.1),
            (FX, "F 4", 0.1),
        ]));

    let mut phrase = Phrase::new(
        beat_time_base,
        vec![
            Box::new(kick_pattern),
            Box::new(snare_pattern),
            Box::new(hihat_rhythm),
            Box::new(bass_pattern),
            Box::new(synth_pattern),
            Box::new(fx_pattern),
        ],
    );

    // emit notes and feed them into the player
    let print_event = |sample_time: SampleTime, event: &Option<Event>| {
        println!(
            "{:.1} ({:08}) -> {}",
            sample_time as f64 / beat_time_base.samples_per_beat(),
            sample_time,
            match event {
                Some(event) => {
                    format!("{:?}", event)
                }
                None => "---".to_string(),
            }
        );
    };

    // delay initial playback a bit until we're emitting: the player is already running
    let playback_delay_in_samples =
        player.output_sample_frame_position() + beat_time_base.seconds_to_samples(0.5);

    let play_event = |player: &mut AudioFilePlayer,
                      sample_time: SampleTime,
                      event: &Option<Event>| {
        // play
        if let Event::NoteEvents(notes) = event.as_ref().unwrap_or(&Event::NoteEvents(Vec::new())) {
            for note in notes {
                if let Some(instrument) = note.instrument {
                    if let Some(sample) = sample_pool.get(&instrument) {
                        let mut new_source = sample.clone();
                        new_source.set_volume(note.velocity);
                        player
                            .play_file_source(
                                new_source,
                                speed_from_note(note.note as u8),
                                Some(sample_time as u64 + playback_delay_in_samples),
                            )
                            .unwrap();
                    }
                }
            }
        }
    };

    let mut emitted_sample_time: u64 = 0;
    loop {
        const PRELOAD_SECONDS: f64 = 2.0;

        let seconds_emitted = beat_time_base.samples_to_seconds(emitted_sample_time);
        let seconds_played =
            beat_time_base.samples_to_seconds(player.output_sample_frame_position());
        let seconds_to_emit = seconds_played - seconds_emitted + PRELOAD_SECONDS;

        if seconds_to_emit > 1.0 {
            emitted_sample_time += beat_time_base.seconds_to_samples(seconds_to_emit);
            phrase.run_until_time(emitted_sample_time, |sample_time, event| {
                print_event(sample_time, event);
                play_event(&mut player, sample_time, event);
            });
        } else {
            let sleep_amount = 1.0 - seconds_to_emit;
            std::thread::sleep(std::time::Duration::from_secs_f64(sleep_amount));
        }
    }
}
