use afplay::{AudioFilePlayer, AudioOutput, DefaultAudioOutput};

use afseq::{prelude::InstrumentId, rhythm::beat_time::BeatTimeRhythm};
use rhai::{Dynamic, Engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create player
    let audio_output = DefaultAudioOutput::open()?;
    let player = AudioFilePlayer::new(audio_output.sink(), None);

    // set global state (TODO: should not be global but an engine state)
    const INSTRUMENT_ID: InstrumentId = 22;
    bindings::set_global_state(player.output_sample_rate(), INSTRUMENT_ID);

    // create engine and register bindings
    let mut engine = Engine::new();
    bindings::register(&mut engine);

    /*println!("Functions registered:");
    engine
        .gen_fn_signatures(false)
        .into_iter()
        .for_each(|func| println!("{}", func));
    println!();*/

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

// --------------------------------------------------------------------------------------------------

mod bindings {
    use std::sync::atomic::{AtomicU32, AtomicUsize};

    use afseq::prelude::*;
    use afseq::{event::fixed::FixedEventIter, rhythm::beat_time::BeatTimeRhythm, BeatTimeBase};

    use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, FLOAT, INT};

    static SAMPLE_RATE: AtomicU32 = AtomicU32::new(44100);
    static INSTRUMENT_ID: AtomicUsize = AtomicUsize::new(44100);

    // ---------------------------------------------------------------------------------------------

    pub fn set_global_state(sample_rate: u32, instrument_id: InstrumentId) {
        SAMPLE_RATE.store(sample_rate, std::sync::atomic::Ordering::Relaxed);
        INSTRUMENT_ID.store(instrument_id, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn register(engine: &mut Engine) {
        // Global
        engine
            .register_fn("beat_time", beat_time)
            .register_fn("note", note);

        // BeatTime
        engine
            .register_fn("every_nth_step", every_nth_step)
            .register_fn("every_nth_sixteenth", every_nth_sixteenth)
            .register_fn("every_nth_eighth", every_nth_eighth)
            .register_fn("every_nth_beat", every_nth_beat)
            .register_fn("every_nth_bar", every_nth_bar);

        // BeatTimeRhythm
        engine
            .register_fn("with_pattern", with_pattern)
            .register_fn("trigger", trigger_fixed);
    }

    // ---------------------------------------------------------------------------------------------
    // Global

    fn beat_time(
        beats_per_min: Dynamic,
        beats_per_bar: Dynamic,
    ) -> Result<BeatTimeBase, Box<EvalAltResult>> {
        let bpm = unwrap_float(beats_per_min, "beats_per_min", "beat_time")? as f32;
        let bpb = unwrap_integer(beats_per_bar, "beats_per_bar", "beat_time")? as u32;
        Ok(BeatTimeBase {
            beats_per_min: bpm,
            beats_per_bar: bpb,
            samples_per_sec: SAMPLE_RATE.load(std::sync::atomic::Ordering::Relaxed),
        })
    }

    fn note(s: ImmutableString, velocity: FLOAT) -> FixedEventIter {
        let instrument = INSTRUMENT_ID.load(std::sync::atomic::Ordering::Relaxed);
        new_note_event(instrument, Note::from(s.as_str()), velocity as f32)
    }

    // ---------------------------------------------------------------------------------------------
    // BeatTime

    fn every_nth_step(beat_time: &mut BeatTimeBase, step: BeatTimeStep) -> BeatTimeRhythm {
        beat_time.every_nth_step(step)
    }

    fn every_nth_sixteenth(
        beat_time: &mut BeatTimeBase,
        step_value: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let step = unwrap_float(step_value, "step", "every_nth_beat")? as f32;
        Ok(beat_time.every_nth_sixteenth(step))
    }

    fn every_nth_eighth(
        beat_time: &mut BeatTimeBase,
        step_value: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let step = unwrap_float(step_value, "step", "every_nth_beat")? as f32;
        Ok(beat_time.every_nth_eighth(step))
    }

    fn every_nth_beat(
        beat_time: &mut BeatTimeBase,
        step_value: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let step = unwrap_float(step_value, "step", "every_nth_beat")? as f32;
        Ok(beat_time.every_nth_beat(step))
    }

    fn every_nth_bar(
        beat_time: &mut BeatTimeBase,
        step_value: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let step = unwrap_float(step_value, "step", "every_nth_beat")? as f32;
        Ok(beat_time.every_nth_bar(step))
    }

    // ---------------------------------------------------------------------------------------------
    // BeatTimeRhythm

    fn with_pattern(
        rhythm: &mut BeatTimeRhythm,
        pattern: rhai::Array,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let mut vec = Vec::with_capacity(pattern.len());
        for e in pattern {
            vec.push(unwrap_integer(e, "array element", "with_pattern")?)
        }
        Ok(rhythm.with_pattern_vector(vec))
    }

    fn trigger_fixed(rhythm: &mut BeatTimeRhythm, event: FixedEventIter) -> BeatTimeRhythm {
        rhythm.trigger::<FixedEventIter>(event)
    }

    // ---------------------------------------------------------------------------------------------
    // Binding helpers

    fn unwrap_float(
        d: Dynamic,
        arg_name: &str,
        func_name: &str,
    ) -> Result<FLOAT, Box<EvalAltResult>> {
        if let Some(float) = d.clone().try_cast::<FLOAT>() {
            Ok(float)
        } else if let Some(integer) = d.try_cast::<INT>() {
            Ok(integer as f64)
        } else {
            Err(format!(
                "Invalid arg: '{}' in '{}' must be a number value",
                arg_name, func_name
            )
            .as_str()
            .into())
        }
    }

    fn unwrap_integer(
        d: Dynamic,
        arg_name: &str,
        func_name: &str,
    ) -> Result<INT, Box<EvalAltResult>> {
        if let Some(float) = d.clone().try_cast::<FLOAT>() {
            Ok(float as INT)
        } else if let Some(integer) = d.try_cast::<INT>() {
            Ok(integer)
        } else {
            Err(format!(
                "Invalid arg: '{}' in '{}' must be a number value",
                arg_name, func_name
            )
            .as_str()
            .into())
        }
    }
}
