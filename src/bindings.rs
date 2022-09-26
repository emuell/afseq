use std::sync::atomic::{AtomicU32, AtomicUsize};

use crate::prelude::*;
use crate::{event::fixed::FixedEventIter, rhythm::beat_time::BeatTimeRhythm, BeatTimeBase};

use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, FLOAT, INT};

static SAMPLE_RATE: AtomicU32 = AtomicU32::new(44100);
static INSTRUMENT_ID: AtomicUsize = AtomicUsize::new(usize::MAX);

// ---------------------------------------------------------------------------------------------

pub fn set_global_binding_state<I: Into<Option<InstrumentId>>>(sample_rate: u32, instrument_id: I) {
    SAMPLE_RATE.store(sample_rate, std::sync::atomic::Ordering::Relaxed);

    let instrument_id = instrument_id.into();
    INSTRUMENT_ID.store(
        instrument_id.unwrap_or(usize::MAX),
        std::sync::atomic::Ordering::Relaxed,
    );
}

// ---------------------------------------------------------------------------------------------

pub fn register_bindings(engine: &mut Engine) {
    // Global
    engine
        .register_fn("beat_time", beat_time)
        .register_fn("note", note);

    // BeatTime
    engine
        .register_fn("every_nth_sixteenth", every_nth_sixteenth)
        .register_fn("every_nth_eighth", every_nth_eighth)
        .register_fn("every_nth_beat", every_nth_beat)
        .register_fn("every_nth_bar", every_nth_bar);

    // BeatTimeRhythm
    engine
        .register_fn("with_pattern", with_pattern)
        .register_fn("with_offset", with_offset)
        .register_fn("trigger", trigger_fixed_event);
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
    if instrument == usize::MAX {
        new_note_event(None, Note::from(s.as_str()), velocity as f32)
    } else {
        new_note_event(instrument, Note::from(s.as_str()), velocity as f32)
    }
}

// ---------------------------------------------------------------------------------------------
// BeatTime

fn every_nth_sixteenth(
    this: &mut BeatTimeBase,
    sixteenth: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(sixteenth, "step", "every_nth_beat")? as f32;
    Ok(this.every_nth_sixteenth(step))
}

fn every_nth_eighth(
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(beats, "step", "every_nth_beat")? as f32;
    Ok(this.every_nth_eighth(step))
}

fn every_nth_beat(
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(beats, "step", "every_nth_beat")? as f32;
    Ok(this.every_nth_beat(step))
}

fn every_nth_bar(
    this: &mut BeatTimeBase,
    bars: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(bars, "step", "every_nth_beat")? as f32;
    Ok(this.every_nth_bar(step))
}

// ---------------------------------------------------------------------------------------------
// BeatTimeRhythm

fn with_pattern(
    this: &mut BeatTimeRhythm,
    pattern: rhai::Array,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let mut vec = Vec::with_capacity(pattern.len());
    for e in pattern {
        vec.push(unwrap_integer(e, "array element", "with_pattern")?)
    }
    Ok(this.with_pattern_vector(vec))
}

fn with_offset(
    this: &mut BeatTimeRhythm,
    offset: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let offset = unwrap_float(offset, "offset", "with_offset")? as f32;
    Ok(this.with_offset_in_step(offset))
}

fn trigger_fixed_event(this: &mut BeatTimeRhythm, event: FixedEventIter) -> BeatTimeRhythm {
    this.trigger::<FixedEventIter>(event)
}

// ---------------------------------------------------------------------------------------------
// Binding helpers

fn unwrap_float(d: Dynamic, arg_name: &str, func_name: &str) -> Result<FLOAT, Box<EvalAltResult>> {
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

fn unwrap_integer(d: Dynamic, arg_name: &str, func_name: &str) -> Result<INT, Box<EvalAltResult>> {
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

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        event::{fixed::FixedEventIter, Event, Note},
        prelude::BeatTimeStep,
        rhythm::beat_time::BeatTimeRhythm,
        BeatTimeBase,
    };

    use super::{register_bindings, set_global_binding_state};
    use rhai::{Dynamic, Engine};

    #[test]
    fn beat_time() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        set_global_binding_state(4800, None);
        register_bindings(&mut engine);

        // BeatTime
        assert!(engine
            // int -> float, float -> int casts
            .eval::<Dynamic>("beat_time(120, 4.0)",)
            .unwrap()
            .try_cast::<BeatTimeBase>()
            .is_some());
        assert!(engine
            // str -> int should fail
            .eval::<Dynamic>(r#"beat_time(120.0, "4.0")"#)
            .is_err());

        // NoteEvent
        let note_event = engine
            // int -> float, float -> int casts
            .eval::<Dynamic>(r#"note("C#1", 0.5)"#)
            .unwrap()
            .try_cast::<FixedEventIter>();
        assert!(
            if let Event::NoteEvents(notes) = note_event.unwrap().event {
                notes.len() == 1 && notes[0].note == Note::from("C#1") && notes[0].velocity == 0.5
            } else {
                false
            }
        );

        // BeatTimeRhythm
        let beat_time_rhythm = engine
            .eval::<Dynamic>(
                r#"
                  beat_time(120.0, 4.0)
                    .every_nth_beat(1)
                    .with_offset(2)
                    .with_pattern([1,0,1,0]);
                "#,
            )
            .unwrap()
            .try_cast::<BeatTimeRhythm>();
        assert!(beat_time_rhythm.is_some());
        assert_eq!(
            beat_time_rhythm.clone().unwrap().step(),
            BeatTimeStep::Beats(1.0)
        );
        assert_eq!(
            beat_time_rhythm.clone().unwrap().offset(),
            BeatTimeStep::Beats(2.0)
        );
        assert_eq!(
            beat_time_rhythm.unwrap().pattern(),
            vec![true, false, true, false]
        );
    }
}
