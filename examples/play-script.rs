use lazy_static::*;

use std::{
    collections::HashMap,
    path::Path,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use afplay::{
    source::file::preloaded::PreloadedFileSource, utils::speed_from_note, AudioFilePlayer,
    AudioOutput, DefaultAudioOutput, FilePlaybackOptions,
};

use afseq::{
    bindings::{self},
    prelude::*,
    rhythm::beat_time::BeatTimeRhythm,
};

use notify::{RecursiveMode, Watcher};
use rhai::{Dynamic, EvalAltResult, NativeCallContext};

// -------------------------------------------------------------------------------------------------

#[derive(Clone)]
struct SamplePool {
    samples: Arc<RwLock<HashMap<InstrumentId, Arc<PreloadedFileSource>>>>,
}

impl SamplePool {
    fn new() -> Self {
        Self {
            samples: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_sample(&self, id: InstrumentId) -> Option<PreloadedFileSource> {
        self.samples
            .read()
            .unwrap()
            .get(&id)
            .map(|sample| sample.as_ref().clone())
    }

    fn load_sample(&self, file_path: &str) -> Result<InstrumentId, Box<dyn std::error::Error>> {
        let sample = PreloadedFileSource::new(file_path, None, FilePlaybackOptions::default())?;
        let id = unique_instrument_id();
        self.samples.write().unwrap().insert(id, Arc::new(sample));
        Ok(id)
    }
}

// -------------------------------------------------------------------------------------------------

// load and run a single script and return the rhytm created by the script or an error
fn load_script(
    sample_pool: &'static SamplePool,
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Result<Box<dyn Rhythm>, Box<dyn std::error::Error>> {
    // create a new engine
    let mut engine = bindings::new_engine();
    bindings::register(&mut engine, time_base, Some(instrument));

    // register sample pool API
    engine.register_fn("sample_pool", move || -> &'static SamplePool {
        sample_pool
    });
    engine.register_type::<SamplePool>().register_fn(
        "load",
        |context: NativeCallContext,
         this: &SamplePool,
         file_path: String|
         -> Result<rhai::INT, Box<EvalAltResult>> {
            match this.load_sample(&file_path) {
                Ok(id) => Ok(*id as rhai::INT),
                Err(_err) => {
                    Err(EvalAltResult::ErrorModuleNotFound(file_path, context.position()).into())
                }
            }
        },
    );

    // compile and evaluate script
    let ast = engine.compile_file(PathBuf::from(file_name))?;
    let result = engine.eval_ast::<Dynamic>(&ast)?;

    // hande script result
    if let Some(beat_time_rhythm) = result.clone().try_cast::<BeatTimeRhythm>() {
        Ok(Box::new(beat_time_rhythm))
    } else {
        Err(EvalAltResult::ErrorMismatchDataType(
            "Rhythm".to_string(),
            result.type_name().to_string(),
            rhai::Position::new(1, 1),
        )
        .into())
    }
}

// -------------------------------------------------------------------------------------------------

fn load_script_with_fallback(
    sample_pool: &'static SamplePool,
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Box<dyn Rhythm> {
    load_script(sample_pool, instrument, time_base, file_name).unwrap_or_else(|err| {
        println!("script '{}' failed to compile: {}", file_name, err);
        Box::new(BeatTimeRhythm::new(time_base, BeatTimeStep::Beats(1.0)))
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let mut player = AudioFilePlayer::new(audio_output.sink(), None);

    // create sample pool
    lazy_static! {
        static ref POOL: SamplePool = SamplePool::new();
    }
    let KICK: InstrumentId = POOL.load_sample("assets/kick.wav")?;
    let SNARE: InstrumentId = POOL.load_sample("assets/snare.wav")?;
    let HIHAT: InstrumentId = POOL.load_sample("assets/hat.wav")?;
    let BASS: InstrumentId = POOL.load_sample("assets/bass.wav")?;
    let _SYNTH: InstrumentId = POOL.load_sample("assets/synth.wav")?;
    let TONE: InstrumentId = POOL.load_sample("assets/tone.wav")?;
    let _FX: InstrumentId = POOL.load_sample("assets/fx.wav")?;

    // set default time base config
    let beat_time = BeatTimeBase {
        beats_per_min: 120.0,
        beats_per_bar: 4,
        samples_per_sec: player.output_sample_rate(),
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
            beat_time,
            vec![
                load_script_with_fallback(&POOL, KICK, beat_time, "./assets/scripts/kick.rhai"),
                load_script_with_fallback(&POOL, SNARE, beat_time, "./assets/scripts/snare.rhai"),
                load_script_with_fallback(&POOL, HIHAT, beat_time, "./assets/scripts/hihat.rhai"),
                load_script_with_fallback(&POOL, BASS, beat_time, "./assets/scripts/bass.rhai"),
                load_script_with_fallback(&POOL, TONE, beat_time, "./assets/scripts/tone.rhai"),
            ],
        );

        // schedule events relative to the player's initial position
        let playback_start_in_samples = player.output_sample_frame_position();
        // emit notes and feed them into the player
        let play_event = |player: &mut AudioFilePlayer,
                          _rhythm_index: usize,
                          sample_time: SampleTime,
                          event: &Option<Event>| {
            // play
            if let Some(Event::NoteEvents(notes)) = event {
                for (_voice_index, note) in notes.iter().enumerate() {
                    if let Some(note) = note {
                        // TODO: stop playing note at (rhythm_index, voice_index) column
                        if note.note.is_note_on() {
                            if let Some(instrument) = note.instrument {
                                if let Some(mut sample) = POOL.get_sample(instrument) {
                                    sample.set_volume(note.velocity);
                                    player
                                        .play_file_source(
                                            sample,
                                            speed_from_note(note.note as u8),
                                            Some(sample_time as u64 + playback_start_in_samples),
                                        )
                                        .unwrap();
                                }
                            }
                        }
                    }
                }
            }
        };

        let mut emitted_sample_time: u64 = 0;
        loop {
            const PRELOAD_SECONDS: f64 = 2.0;

            let seconds_emitted = beat_time.samples_to_seconds(emitted_sample_time);
            let seconds_played = beat_time.samples_to_seconds(
                player.output_sample_frame_position() - playback_start_in_samples,
            );
            let seconds_to_emit = seconds_played - seconds_emitted + PRELOAD_SECONDS;

            if seconds_to_emit > 1.0 {
                emitted_sample_time += beat_time.seconds_to_samples(seconds_to_emit);
                phrase.run_until_time(emitted_sample_time, |rhythm_index, sample_time, event| {
                    // print_event(sample_time, event);
                    play_event(&mut player, rhythm_index, sample_time, event);
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
