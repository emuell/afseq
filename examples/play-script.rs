use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use afseq::prelude::*;

use notify::{RecursiveMode, Watcher};

// -------------------------------------------------------------------------------------------------

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create event player
    let mut player = SamplePlayer::new(None)?;

    // preload samples
    let KICK = player.sample_pool().load_sample("assets/kick.wav")?;
    let SNARE = player.sample_pool().load_sample("assets/snare.wav")?;
    let HIHAT = player.sample_pool().load_sample("assets/hihat.wav")?;
    let BASS = player.sample_pool().load_sample("assets/bass.wav")?;
    let SYNTH = player.sample_pool().load_sample("assets/synth.wav")?;
    let TONE = player.sample_pool().load_sample("assets/tone.wav")?;
    let FX = player.sample_pool().load_sample("assets/fx.wav")?;

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
            bindings::new_rhythm_from_script(
                id,
                beat_time,
                format!("./assets/{file_name}").as_str(),
            )
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
