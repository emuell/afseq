//! Rhai script bindings for the entire crate.

use crate::prelude::*;
use crate::{event::fixed::FixedEventIter, rhythm::beat_time::BeatTimeRhythm, BeatTimeBase};

use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, NativeCallContext, FLOAT, INT};

// ---------------------------------------------------------------------------------------------

pub fn register_bindings(
    engine: &mut Engine,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) {
    // Defaults
    engine
        .register_fn("__default_instrument", move || default_instrument)
        .register_fn("__default_beat_time", move || default_time_base);

    // Global
    engine
        .register_fn("beat_time", default_beat_time)
        .register_fn("beat_time", beat_time)
        .register_fn("note", note)
        .register_fn("note", note_vec)
        .register_fn("note_seq", note_vec_seq);

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

fn default_beat_time(context: NativeCallContext) -> Result<BeatTimeBase, Box<EvalAltResult>> {
    eval_default_beat_time(context.engine())
}

fn beat_time(
    context: NativeCallContext,
    beats_per_min: Dynamic,
    beats_per_bar: Dynamic,
) -> Result<BeatTimeBase, Box<EvalAltResult>> {
    let default_beat_time = eval_default_beat_time(context.engine())?;
    let bpm = unwrap_float(&context, beats_per_min, "beats_per_min")? as f32;
    let bpb = unwrap_integer(&context, beats_per_bar, "beats_per_bar")? as u32;
    Ok(BeatTimeBase {
        beats_per_min: bpm,
        beats_per_bar: bpb,
        samples_per_sec: default_beat_time.samples_per_second(),
    })
}

fn note(
    context: NativeCallContext,
    string: ImmutableString,
    velocity: FLOAT,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let instrument = eval_default_instrument(context.engine())?;
    Ok(new_note_event(
        instrument,
        unwrap_note_from_string(&context, string.as_str())?,
        velocity as f32,
    ))
}

fn note_vec(
    context: NativeCallContext,
    array: Array,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let instrument = eval_default_instrument(context.engine())?;
    let mut sequence = Vec::with_capacity(array.len());
    for item in array {
        let note_item_array = item.into_array()?;
        if note_item_array.len() != 2 {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Expected 2 items in note array in function '{}', got '{}' items",
                    context.fn_name(),
                    note_item_array.len()
                )
                .into(),
                context.position(),
            )
            .into());
        }
        let note = unwrap_note(&context, note_item_array[0].clone())?;
        let velocity = unwrap_float(&context, note_item_array[1].clone(), "velocity")? as f32;
        sequence.push((instrument, note, velocity));
    }
    Ok(new_polyphonic_note_event(sequence))
}

fn note_vec_seq(
    context: NativeCallContext,
    array: Array,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    // NB: array arg may be a:
    // [[NOTE, VEL], ..] -> sequence of single notes
    // [[[NOTE, VEL], ..], [[NOTE, VEL]]] -> sequence of poly notes
    let instrument = eval_default_instrument(context.engine())?;
    let mut event_sequence = Vec::with_capacity(array.len());
    for item1_dyn in array {
        let item1_array_result = item1_dyn.into_array();
        if let Err(other_type) = item1_array_result {
            return Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Expected 'array' arg in function '{}', got '{}'",
                    context.fn_name(),
                    other_type
                )
                .into(),
                context.position(),
            )
            .into());
        }
        let item1_arr = item1_array_result.unwrap();
        let mut note_events = Vec::with_capacity(item1_arr.len());
        if !item1_arr.is_empty() && item1_arr[0].type_name() == "string" {
            // Vec<Vec<NOTE, VEL>>
            if item1_arr.len() != 2 {
                return Err(EvalAltResult::ErrorInModule(
                    "bindings".to_string(),
                    format!(
                        "Expected 2 items in note array in function '{}', got '{}' items",
                        context.fn_name(),
                        item1_arr.len()
                    )
                    .into(),
                    context.position(),
                )
                .into());
            }
            let note = unwrap_note(&context, item1_arr[0].clone())?;
            let velocity = unwrap_float(&context, item1_arr[1].clone(), "velocity")? as f32;
            note_events.push((instrument, note, velocity));
        } else {
            // Vec<Vec<Vec<NOTE, VEL>>>
            for item2_dyn in item1_arr {
                let item2_array_result = item2_dyn.into_array();
                if let Err(other_type) = item2_array_result {
                    return Err(format!("Expected 'array', got '{}'", other_type).into());
                }
                let item2_arr = item2_array_result.unwrap();
                if item2_arr.len() != 2 {
                    return Err(format!(
                        "Expected 2 items in note array ['note', 'velocity'], got '{}' items",
                        item2_arr.len()
                    )
                    .into());
                }
                let note = unwrap_note(&context, item2_arr[0].clone())?;
                let velocity = unwrap_float(&context, item2_arr[1].clone(), "velocity")? as f32;
                note_events.push((instrument, note, velocity));
            }
        }
        event_sequence.push(note_events)
    }
    Ok(new_polyphonic_note_sequence_event(event_sequence))
}

// ---------------------------------------------------------------------------------------------
// BeatTime

fn every_nth_sixteenth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    sixteenth: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(&context, sixteenth, "step")? as f32;
    Ok(this.every_nth_sixteenth(step))
}

fn every_nth_eighth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(&context, beats, "step")? as f32;
    Ok(this.every_nth_eighth(step))
}

fn every_nth_beat(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(&context, beats, "step")? as f32;
    Ok(this.every_nth_beat(step))
}

fn every_nth_bar(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    bars: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let step = unwrap_float(&context, bars, "step")? as f32;
    Ok(this.every_nth_bar(step))
}

// ---------------------------------------------------------------------------------------------
// BeatTimeRhythm

fn with_pattern(
    context: NativeCallContext,
    this: &mut BeatTimeRhythm,
    pattern: Array,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let mut vec = Vec::with_capacity(pattern.len());
    for e in pattern {
        vec.push(unwrap_integer(&context, e, "array element")?)
    }
    Ok(this.with_pattern_vector(vec))
}

fn with_offset(
    context: NativeCallContext,
    this: &mut BeatTimeRhythm,
    offset: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let offset = unwrap_float(&context, offset, "offset")? as f32;
    Ok(this.with_offset_in_step(offset))
}

fn trigger_fixed_event(this: &mut BeatTimeRhythm, event: FixedEventIter) -> BeatTimeRhythm {
    this.trigger(event)
}

// ---------------------------------------------------------------------------------------------
// Binding helpers

fn eval_default_instrument(engine: &Engine) -> Result<Option<InstrumentId>, Box<EvalAltResult>> {
    engine.eval_expression::<Option<InstrumentId>>("__default_instrument()")
}

fn eval_default_beat_time(engine: &Engine) -> Result<BeatTimeBase, Box<EvalAltResult>> {
    engine.eval_expression::<BeatTimeBase>("__default_beat_time()")
}

fn unwrap_float(
    context: &NativeCallContext,
    d: Dynamic,
    arg_name: &str,
) -> Result<FLOAT, Box<EvalAltResult>> {
    if let Some(float) = d.clone().try_cast::<FLOAT>() {
        Ok(float)
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer as f64)
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
                arg_name,
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

fn unwrap_integer(
    context: &NativeCallContext,
    d: Dynamic,
    arg_name: &str,
) -> Result<INT, Box<EvalAltResult>> {
    if let Some(float) = d.clone().try_cast::<FLOAT>() {
        Ok(float as INT)
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer)
    } else {
        Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
                arg_name,
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
    }
}

fn unwrap_note(context: &NativeCallContext, d: Dynamic) -> Result<Note, Box<EvalAltResult>> {
    match d.into_string() {
        Ok(s) => unwrap_note_from_string(context, s.as_str()),
        Err(other_type) => Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': Argument is not a string, but a '{}'.",
                context.fn_name(),
                other_type
            )
            .into(),
            context.position(),
        )
        .into()),
    }
}

fn unwrap_note_from_string(
    context: &NativeCallContext,
    s: &str,
) -> Result<Note, Box<EvalAltResult>> {
    match Note::try_from(s) {
        Ok(note) => Ok(note),
        Err(err) => Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': {}",
                context.fn_name(),
                err
            )
            .into(),
            context.position(),
        )
        .into()),
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        bindings::eval_default_instrument,
        event::{fixed::FixedEventIter, Event, Note},
        prelude::BeatTimeStep,
        rhythm::beat_time::BeatTimeRhythm,
        BeatTimeBase,
    };

    use super::{eval_default_beat_time, register_bindings};
    use rhai::{Dynamic, Engine};

    #[test]
    fn defaults() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        register_bindings(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
            Some(76),
        );

        assert!(eval_default_beat_time(&engine).is_ok());
        let default_beat_time = eval_default_beat_time(&engine).unwrap();
        assert_eq!(default_beat_time.beats_per_min, 160.0);
        assert_eq!(default_beat_time.beats_per_bar, 6);
        assert_eq!(default_beat_time.samples_per_sec, 96000);

        assert!(eval_default_instrument(&engine).is_ok());
        assert_eq!(eval_default_instrument(&engine).unwrap(), Some(76));
    }

    #[test]
    fn note() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        register_bindings(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
            None,
        );

        // Note
        assert!(engine.eval::<Dynamic>(r#"note("X#1", 0.5)"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note("C#1", "0.5")"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note("C#1", 0.5, 1.0)"#).is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note("C#1", 0.5)"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let note_event = eval_result.unwrap().try_cast::<FixedEventIter>();
            assert!(
                if let Event::NoteEvents(notes) = &note_event.unwrap().events()[0] {
                    notes.len() == 1
                        && notes[0].note == Note::from("C#1")
                        && notes[0].velocity == 0.5
                } else {
                    false
                }
            );
        }

        let eval_result = engine.eval::<Dynamic>(r#"note([["C#1", 0.5], ["G2", 0.75]])"#);
        assert!(engine
            .eval::<Dynamic>(r#"note([["Note", 0.5, 1.0]])"#)
            .is_err());
        assert!(engine
            .eval::<Dynamic>(r#"note([["C#1", 0.5, 1.0]])"#)
            .is_err());
        assert!(engine.eval::<Dynamic>(r#"note([["C#1", "0.5"]])"#).is_err());
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let poly_note_event = eval_result.unwrap().try_cast::<FixedEventIter>();
            assert!(poly_note_event.is_some());
            let note_event = &poly_note_event.unwrap().events()[0];
            assert!(if let Event::NoteEvents(notes) = &note_event {
                notes.len() == 2
                    && notes[0].note == Note::from("C#1")
                    && notes[0].velocity == 0.5
                    && notes[1].note == Note::from("G2")
                    && notes[1].velocity == 0.75
            } else {
                false
            });
        }

        // NoteEventSequence
        let eval_result = engine.eval::<Dynamic>(r#"note_seq([["C#1", 0.5], ["G_2", 0.75]])"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let note_sequence_event = eval_result.unwrap().try_cast::<FixedEventIter>();
            assert!(note_sequence_event.is_some());
            let note_events = note_sequence_event.unwrap().events();
            assert!(if let Event::NoteEvents(notes) = &note_events[0] {
                notes.len() == 1 && notes[0].note == Note::from("C#1") && notes[0].velocity == 0.5
            } else {
                false
            });
            assert!(if let Event::NoteEvents(notes) = &note_events[1] {
                notes.len() == 1 && notes[0].note == Note::from("G2") && notes[0].velocity == 0.75
            } else {
                false
            });
        }
        let poly_note_sequence_event = engine
            .eval::<Dynamic>(
                r#"note_seq([
                     [["C#1", 0.5], ["G_2", 0.75]], 
                     [["A#5", 0.2], ["B_1", 0.1]]
                   ])"#,
            )
            .unwrap()
            .try_cast::<FixedEventIter>();
        assert!(poly_note_sequence_event.is_some());
        let note_events = poly_note_sequence_event.unwrap().events();
        assert!(if let Event::NoteEvents(notes) = &note_events[0] {
            notes.len() == 2
                && notes[0].note == Note::from("C#1")
                && notes[0].velocity == 0.5
                && notes[1].note == Note::from("G2")
                && notes[1].velocity == 0.75
        } else {
            false
        });
        assert!(if let Event::NoteEvents(notes) = &note_events[1] {
            notes.len() == 2
                && notes[0].note == Note::from("A#5")
                && notes[0].velocity == 0.2
                && notes[1].note == Note::from("B1")
                && notes[1].velocity == 0.1
        } else {
            false
        });
    }

    #[test]
    fn beat_time() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        register_bindings(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
            None,
        );

        // BeatTime
        assert!(engine
            .eval::<Dynamic>("beat_time()",)
            .unwrap()
            .try_cast::<BeatTimeBase>()
            .is_some());
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
