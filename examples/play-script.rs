use std::{
    path::Path,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use afseq::prelude::*;

use notify::{RecursiveMode, Watcher};
use rhai::{Dynamic, EvalAltResult};

// -------------------------------------------------------------------------------------------------

// load and run a single script and return a fallback rhythm on errors
fn load_script(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Box<dyn Rhythm> {
    let do_load = || -> Result<Box<dyn Rhythm>, Box<dyn std::error::Error>> {
        // create a new engine
        let mut engine = bindings::new_engine();
        bindings::register(&mut engine, time_base, Some(instrument));

        // compile and evaluate script
        let ast = engine.compile_file(PathBuf::from(file_name))?;
        let result = engine.eval_ast::<Dynamic>(&ast)?;

        // hande script result
        if let Some(beat_time_rhythm) = result.clone().try_cast::<BeatTimeRhythm>() {
            Ok(Box::new(beat_time_rhythm))
        } else if let Some(second_time_rhythm) = result.clone().try_cast::<SecondTimeRhythm>() {
            Ok(Box::new(second_time_rhythm))
        } else {
            Err(EvalAltResult::ErrorMismatchDataType(
                "Rhythm".to_string(),
                result.type_name().to_string(),
                rhai::Position::new(1, 1),
            )
            .into())
        }
    };

    do_load().unwrap_or_else(|err| {
        println!("script '{}' failed to compile: {}", file_name, err);
        Box::new(BeatTimeRhythm::new(time_base, BeatTimeStep::Beats(1.0)))
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create event player
    let mut player = SamplePlayer::new()?;

    // preload samples
    let KICK = player.load_sample("assets/kick.wav")?;
    let SNARE = player.load_sample("assets/snare.wav")?;
    let HIHAT = player.load_sample("assets/hihat.wav")?;
    let BASS = player.load_sample("assets/bass.wav")?;
    let SYNTH = player.load_sample("assets/synth.wav")?;
    let TONE = player.load_sample("assets/tone.wav")?;
    let FX = player.load_sample("assets/fx.wav")?;

    // set default time base config
    let beat_time = BeatTimeBase {
        beats_per_min: 124.0,
        beats_per_bar: 4,
        samples_per_sec: player.sample_rate(),
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
    watcher.watch(Path::new("./assets"), RecursiveMode::Recursive)?;

    // (re)run all scripts
    loop {
        // build final phrase
        let load = |id: InstrumentId, file_name: &str| {
            load_script(id, beat_time, format!("./assets/{file_name}").as_str())
        };
        let mut phrase = Phrase::new(
            beat_time,
            vec![
                load(KICK, "kick.rhai"),
                load(SNARE, "snare.rhai"),
                load(HIHAT, "hihat.rhai"),
                load(BASS, "bass.rhai"),
                load(SYNTH, "synth.rhai"),
                load(TONE, "tone.rhai"),
                load(FX, "fx.rhai"),
            ],
        );

        player.run_until(&mut phrase, &beat_time, {
            let script_files_changed = script_files_changed.clone();
            move || {
                if script_files_changed.load(Ordering::Relaxed) {
                    script_files_changed.store(false, Ordering::Relaxed);
                    println!("Rebuilding all rhythms...");
                    true
                } else {
                    false
                }
            }
        });
    }
}
