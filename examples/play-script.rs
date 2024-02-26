use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use notify::{RecursiveMode, Watcher};
use simplelog::*;

use afseq::prelude::*;

// -------------------------------------------------------------------------------------------------

#[cfg(feature = "dhat-profiler")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// -------------------------------------------------------------------------------------------------

// TODO: make this configurable with an cmd line arg
const DEMO_PATH: &str = "./examples/assets";

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

    // fetch contents from demo dir
    log::info!("Searching for wav/script files in path '{}'...", DEMO_PATH);
    let mut entry_stems = HashSet::<String>::new();
    let paths = fs::read_dir(DEMO_PATH).expect("Failed to access demo content directory");
    for path in paths {
        let path = path?.path();
        if let Some(extension) = path.extension() {
            let extension = extension.to_string_lossy().to_string();
            if matches!(extension.as_str(), "lua" | "wav") {
                if let Some(stem) = path.file_stem() {
                    entry_stems.insert(stem.to_string_lossy().to_string());
                }
            }
        }
    }

    // load samples and get paths to the rhythm scripts
    let sample_pool = SamplePool::new();
    struct RhythmEntry {
        instrument_id: InstrumentId,
        script_path: String,
    }
    let mut entries = vec![];
    for stem in entry_stems.iter() {
        let base_path = PathBuf::new().join(DEMO_PATH).join(stem);
        let wave_file = base_path.with_extension("wav");
        let lua_file = base_path.with_extension("lua");
        if wave_file.exists() && lua_file.exists() {
            log::info!("Found file/script: '{}'...", stem);
            let instrument_id = sample_pool.load_sample(&wave_file.to_string_lossy())?;
            let script_path = lua_file.to_string_lossy().to_string();
            entries.push(RhythmEntry {
                instrument_id,
                script_path,
            });
        } else if lua_file.exists() || wave_file.exists() {
            log::warn!("Ignoring file/script: '{}'...", stem);
        }
    }

    // create event player
    let mut player = SamplePlayer::new(Arc::new(RwLock::new(sample_pool)), None)?;

    // set default time base config
    let beat_time = BeatTimeBase {
        beats_per_min: 124.0,
        beats_per_bar: 4,
        samples_per_sec: player.file_player().output_sample_rate(),
    };

    // Watch for script changes, signaling in 'script_files_changed'
    let script_files_changed = Arc::new(AtomicBool::new(false));

    let mut watcher = notify::recommended_watcher({
        let script_files_changed = script_files_changed.clone();
        move |res| match res {
            Ok(event) => {
                log::info!("File change event: {:?}", event);
                script_files_changed.store(true, Ordering::Relaxed);
            }
            Err(err) => log::error!("File watch error: {}", err),
        }
    })?;
    watcher.watch(Path::new(DEMO_PATH), RecursiveMode::Recursive)?;

    // stop on Control-C
    let stop_running = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler({
        let stop_running = stop_running.clone();
        move || {
            stop_running.store(true, Ordering::Relaxed);
        }
    })?;

    // (re)run all scripts
    while !stop_running.load(Ordering::Relaxed) {
        if script_files_changed.load(Ordering::Relaxed) {
            script_files_changed.store(false, Ordering::Relaxed);
            log::info!("Rebuilding all rhythms...");
        }

        // build final phrase
        let load = |instrument: Option<InstrumentId>, file_name: &str| {
            bindings::new_rhythm_from_file_with_fallback(beat_time, instrument, file_name)
        };
        let phrase = Phrase::new(
            beat_time,
            entries
                .iter()
                .map(|e| load(Some(e.instrument_id), &e.script_path))
                .collect(),
            BeatTimeStep::Bar(4.0),
        );

        // wrap phrase into a sequence
        let mut sequence = Sequence::new(beat_time, vec![phrase]);

        let reset_playback_pos = false;
        player.run_until(&mut sequence, &beat_time, reset_playback_pos, || {
            script_files_changed.load(Ordering::Relaxed) || stop_running.load(Ordering::Relaxed)
        });
    }

    #[cfg(feature = "dhat-profiler")]
    drop(profiler);

    Ok(())
}
