use std::{
    cell::RefCell,
    path::Path,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use afseq::prelude::*;

use notify::{RecursiveMode, Watcher};
use rhai::{Dynamic, EvalAltResult, NativeCallContext};

// -------------------------------------------------------------------------------------------------

// load and run a single script and return a fallback rhythm on errors
fn load_script(
    sample_pool: Rc<RefCell<SamplePool>>,
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Box<dyn Rhythm> {
    let do_load = || -> Result<Box<dyn Rhythm>, Box<dyn std::error::Error>> {
        // create a new engine
        let mut engine = bindings::new_engine();
        bindings::register(&mut engine, time_base, Some(instrument));

        // register sample pool API
        engine.register_fn(
            "load_sample",
            move |context: NativeCallContext,
                  file_path: String|
                  -> Result<rhai::INT, Box<EvalAltResult>> {
                match sample_pool.borrow_mut().load_sample(&file_path) {
                    Ok(id) => Ok(*id as rhai::INT),
                    Err(_err) => Err(EvalAltResult::ErrorModuleNotFound(
                        file_path,
                        context.position(),
                    )
                    .into()),
                }
            },
        );

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
    // create sample pool
    let sample_pool = Rc::new(RefCell::new(SamplePool::new()));

    let KICK = sample_pool.borrow_mut().load_sample("assets/kick.wav")?;
    let SNARE = sample_pool.borrow_mut().load_sample("assets/snare.wav")?;
    let HIHAT = sample_pool.borrow_mut().load_sample("assets/hat.wav")?;
    let BASS = sample_pool.borrow_mut().load_sample("assets/bass.wav")?;
    let SYNTH = sample_pool.borrow_mut().load_sample("assets/synth.wav")?;
    let TONE = sample_pool.borrow_mut().load_sample("assets/tone.wav")?;
    let FX = sample_pool.borrow_mut().load_sample("assets/fx.wav")?;

    // create event player
    let mut player = SamplePlayer::new(sample_pool.clone())?;

    // set default time base config
    let beat_time = BeatTimeBase {
        beats_per_min: 120.0,
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
    watcher.watch(Path::new("./assets/scripts"), RecursiveMode::Recursive)?;

    // (re)run all scripts
    loop {
        // build final phrase
        let mut phrase = Phrase::new(
            beat_time,
            vec![
                load_script(
                    sample_pool.clone(),
                    KICK,
                    beat_time,
                    "./assets/scripts/kick.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    SNARE,
                    beat_time,
                    "./assets/scripts/snare.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    HIHAT,
                    beat_time,
                    "./assets/scripts/hihat.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    BASS,
                    beat_time,
                    "./assets/scripts/bass.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    SYNTH,
                    beat_time,
                    "./assets/scripts/synth.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    TONE,
                    beat_time,
                    "./assets/scripts/tone.rhai",
                ),
                load_script(
                    sample_pool.clone(),
                    FX,
                    beat_time,
                    "./assets/scripts/fx.rhai",
                ),
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
