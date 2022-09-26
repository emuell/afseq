use afplay::{AudioFilePlayer, AudioOutput, DefaultAudioOutput};

use afseq::prelude::*;

use afseq::bindings::{register_bindings, set_global_binding_state};
use afseq::rhythm::beat_time::BeatTimeRhythm;

use rhai::{Dynamic, Engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let player = AudioFilePlayer::new(audio_output.sink(), None);

    // set global state (TODO: should not be global but an engine state)
    const INSTRUMENT_ID: InstrumentId = 22;
    set_global_binding_state(player.output_sample_rate(), INSTRUMENT_ID);

    // create engine and register bindings
    let mut engine = Engine::new();
    register_bindings(&mut engine);

    // run test script
    let result = engine.eval::<Dynamic>(
        r#"
            beat_time(120.0, 4.0)
              .every_nth_beat(1)
              .trigger(note("C4", 1.0))
              .with_pattern([1,0,1,0]);
        "#,
    )?;

    if let Some(rhythm) = result.clone().try_cast::<BeatTimeRhythm>() {
        for e in rhythm.take(16) {
            println!("{:?}", e);
        }
    } else {
        println!("Unexpected script result: {}", result);
    }

    Ok(())
}
