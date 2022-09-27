use afplay::{AudioFilePlayer, AudioOutput, DefaultAudioOutput};

use afseq::prelude::*;

use afseq::bindings::register_bindings;
use afseq::rhythm::beat_time::BeatTimeRhythm;

use rhai::{Dynamic, Engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let player = AudioFilePlayer::new(audio_output.sink(), None);

    // create engine, set defaults and register bindings
    let mut engine = Engine::new();

    let default_instrument = Some(22);
    let default_time_base = BeatTimeBase {
        beats_per_min: 120.0,
        beats_per_bar: 4,
        samples_per_sec: player.output_sample_rate(),
    };

    register_bindings(&mut engine, default_time_base, default_instrument);

    // run test script
    let result = engine.eval::<Dynamic>(
        r#"
            beat_time()
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
