use std::{
    collections::HashMap,
    path::Path,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlayer,
    AudioOutput, DefaultAudioOutput, FilePlaybackOptions,
};

use afseq::{bindings::register_bindings, prelude::*, rhythm::beat_time::BeatTimeRhythm};

use notify::{RecursiveMode, Watcher};
use rhai::{Dynamic, Engine, EvalAltResult};

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

    let load_sample_file = |file_name| {
        PreloadedFileSource::new(file_name, None, FilePlaybackOptions::default()).unwrap()
    };

    let sample_pool: HashMap<InstrumentId, PreloadedFileSource> = HashMap::from([
        (KICK, load_sample_file("assets/kick.wav")),
        (SNARE, load_sample_file("assets/snare.wav")),
        (HIHAT, load_sample_file("assets/hat.wav")),
        (BASS, load_sample_file("assets/bass.wav")),
        (SYNTH, load_sample_file("assets/synth.wav")),
        (FX, load_sample_file("assets/fx.wav")),
    ]);

    // set default time base config
    let beat_time_base = BeatTimeBase {
        beats_per_min: 120.0,
        beats_per_bar: 4,
        samples_per_sec: player.output_sample_rate(),
    };

    // load and run a single script and return the rhytm created by the script or an error
    let load_script = // _
        move |instrument: InstrumentId, file_name: &str|
          -> Result<Box<dyn Rhythm>, Box<dyn std::error::Error>> {
        let mut engine = Engine::new();
        register_bindings(&mut engine, beat_time_base, Some(instrument));
        let result = engine.eval_file::<Dynamic>(PathBuf::from(file_name))?;
        if let Some(beat_time_rhythm) = result.clone().try_cast::<BeatTimeRhythm>() {
            Ok(Box::new(beat_time_rhythm))
        } else {
            Err(EvalAltResult::ErrorMismatchDataType(
                result.type_name().to_string(),
                "Rhythm".to_string(),
                rhai::Position::new(0, 0),
            )
            .into())
        }
    };
    let load_script_with_fallback =
        move |instrument: InstrumentId, file_name: &str| -> Box<dyn Rhythm> {
            load_script(instrument, file_name).unwrap_or_else(|err| {
                println!("script '{}' failed to compile: {}", file_name, err);
                Box::new(BeatTimeRhythm::new(
                    beat_time_base,
                    BeatTimeStep::Beats(1.0),
                ))
            })
        };

    // Watch for script changes, signaling in 'script_files_changed'
    let script_files_changed = Arc::new(AtomicBool::new(false));

    let mut watcher = notify::recommended_watcher({
        let script_files_changed = script_files_changed.clone();
        move |res| match res {
            Ok(event) => {
                println!("File change event: {:?}", event);
                script_files_changed.store(true, Ordering::Relaxed);
            }
            Err(e) => println!("File watch error: {:?}", e),
        }
    })?;
    watcher.watch(Path::new("./assets/scripts"), RecursiveMode::Recursive)?;

    // (re)run all scripts
    loop {
        // stop everything that's playing
        player.stop_all_playing_sources()?;

        // build final phrase
        let mut phrase = Phrase::new(
            beat_time_base,
            vec![
                load_script_with_fallback(KICK, "./assets/scripts/kick.rhai"),
                load_script_with_fallback(SNARE, "./assets/scripts/snare.rhai"),
                load_script_with_fallback(HIHAT, "./assets/scripts/hihat.rhai"),
            ],
        );

        // schedule events relative to the player's initial position
        let playback_start_in_samples = player.output_sample_frame_position();
        // emit notes and feed them into the player
        let play_event =
            |player: &mut AudioFilePlayer, sample_time: SampleTime, event: &Option<Event>| {
                // play
                if let Event::NoteEvents(notes) =
                    event.as_ref().unwrap_or(&Event::NoteEvents(Vec::new()))
                {
                    for note in notes {
                        if let Some(instrument) = note.instrument {
                            if let Some(sample) = sample_pool.get(&instrument) {
                                let mut new_source = sample.clone();
                                new_source.set_volume(note.velocity);
                                player
                                    .play_file_source(
                                        new_source,
                                        speed_from_note(note.note as u8),
                                        Some(sample_time as u64 + playback_start_in_samples),
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
            let seconds_played = beat_time_base.samples_to_seconds(
                player.output_sample_frame_position() - playback_start_in_samples,
            );
            let seconds_to_emit = seconds_played - seconds_emitted + PRELOAD_SECONDS;

            if seconds_to_emit > 1.0 {
                emitted_sample_time += beat_time_base.seconds_to_samples(seconds_to_emit);
                phrase.run_until_time(emitted_sample_time, |sample_time, event| {
                    // print_event(sample_time, event);
                    play_event(&mut player, sample_time, event);
                });
            } else {
                if script_files_changed.load(Ordering::Relaxed) {
                    script_files_changed.store(false, Ordering::Relaxed);
                    println!("Rebuilding all rhythms...");
                    break;
                }
                let sleep_amount = (1.0 - seconds_to_emit).max(0.0);
                std::thread::sleep(std::time::Duration::from_secs_f64(sleep_amount));
            }
        }
    }
}
