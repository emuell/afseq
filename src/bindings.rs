//! Rhai script bindings for the entire crate.

use crate::prelude::*;
use crate::{
    event::{fixed::FixedEventIter, scripted::ScriptedEventIter},
    rhythm::beat_time::BeatTimeRhythm,
    BeatTimeBase,
};

use rhai::{
    packages::Package, Array, Dynamic, Engine, EvalAltResult, FnPtr, ImmutableString,
    NativeCallContext, FLOAT, INT,
};

use rhai_rand::RandomPackage;
use rhai_sci::SciPackage;

use rust_music_theory::{note::Notes, scale};

// ---------------------------------------------------------------------------------------------

pub(crate) mod unwrap;
use unwrap::*;

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_engine() -> Engine {
    let mut engine = Engine::new();

    // Configure engine limits
    engine.set_max_expr_depths(1000, 1000);

    // load default packages
    let sci = SciPackage::new();
    sci.register_into_engine(&mut engine);
    let rand = RandomPackage::new();
    rand.register_into_engine(&mut engine);

    engine
}

// ---------------------------------------------------------------------------------------------

/// Register afseq API bindings into the rhai engine.  
pub fn register(
    engine: &mut Engine,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) {
    // Defaults
    engine
        .register_fn("default_instrument", move || default_instrument)
        .register_fn("default_beat_time", move || default_time_base);

    // Std extensions
    engine.register_fn("repeat", repeat_array);

    // Global
    engine
        .register_fn("beat_time", default_beat_time)
        .register_fn("beat_time", beat_time)
        .register_fn("note", note_from_number)
        .register_fn("note", note_from_string)
        .register_fn("note", note_vec)
        .register_fn("note_seq", note_vec_seq)
        .register_fn("notes_in_scale", notes_in_scale);

    // BeatTime
    engine
        .register_fn("every_nth_sixteenth", every_nth_sixteenth)
        .register_fn("every_sixteenth", every_sixteenth)
        .register_fn("every_nth_eighth", every_nth_eighth)
        .register_fn("every_eighth", every_eighth)
        .register_fn("every_nth_beat", every_nth_beat)
        .register_fn("every_beat", every_beat)
        .register_fn("every_nth_bar", every_nth_bar)
        .register_fn("every_bar", every_bar);

    // BeatTimeRhythm
    engine
        .register_fn("with_pattern", with_pattern)
        .register_fn("with_offset", with_offset)
        .register_fn("trigger", trigger_fixed_event)
        .register_fn("trigger", trigger_custom_event);
}

// ---------------------------------------------------------------------------------------------

fn repeat_array(
    context: NativeCallContext,
    this: Array,
    count: INT,
) -> Result<Array, Box<EvalAltResult>> {
    if count < 0 {
        return Err(EvalAltResult::ErrorArithmetic(
            format!(
                "Count argument in 'array.repeat' must be > 0, but is '{}'",
                count
            ),
            context.position(),
        )
        .into());
    }
    let mut ret = Array::with_capacity(this.len() * count as usize);
    for _ in 0..count {
        for i in this.iter() {
            ret.push(i.clone());
        }
    }
    Ok(ret)
}

// ---------------------------------------------------------------------------------------------
// Defaults

fn eval_default_instrument(engine: &Engine) -> Result<Option<InstrumentId>, Box<EvalAltResult>> {
    engine.eval::<Option<InstrumentId>>("default_instrument()")
}

fn eval_default_beat_time(engine: &Engine) -> Result<BeatTimeBase, Box<EvalAltResult>> {
    engine.eval::<BeatTimeBase>("default_beat_time()")
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
    let err_context = ErrorCallContext::from(&context);
    let default_beat_time = eval_default_beat_time(context.engine())?;
    let bpm = unwrap_float(&err_context, beats_per_min, "beats_per_min")? as f32;
    let bpb = unwrap_integer(&err_context, beats_per_bar, "beats_per_bar")? as u32;
    Ok(BeatTimeBase {
        beats_per_min: bpm,
        beats_per_bar: bpb,
        samples_per_sec: default_beat_time.samples_per_second(),
    })
}

fn note_from_string(
    context: NativeCallContext,
    note: ImmutableString,
    velocity: FLOAT,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let instrument = eval_default_instrument(context.engine())?;
    Ok(new_note_event(
        instrument,
        unwrap_note_from_string(&err_context, note.as_str())?,
        velocity as f32,
    ))
}

fn note_from_number(
    context: NativeCallContext,
    note: INT,
    velocity: FLOAT,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let instrument = eval_default_instrument(context.engine())?;
    Ok(new_note_event(
        instrument,
        unwrap_note_from_int(&err_context, note)?,
        velocity as f32,
    ))
}

fn note_vec(
    context: NativeCallContext,
    array: Array,
) -> Result<FixedEventIter, Box<EvalAltResult>> {
    // NB: array arg may be a:
    // [NOTE, VEL] -> single note
    // [[NOTE, VEL], ..] -> poly notes
    let err_context = ErrorCallContext::from(&context);
    let instrument = eval_default_instrument(context.engine())?;
    let mut sequence = Vec::with_capacity(array.len());
    if !array.is_empty() && (array[0].type_name() == "string" || array[0].is::<INT>()) {
        // [NOTE, VEL]
        sequence.push(unwrap_note_event(&err_context, array, instrument)?);
    } else {
        // [[NOTE, VEL], ..]
        for item in array {
            let note_item_array = unwrap_array(&err_context, item)?;
            sequence.push(unwrap_note_event(
                &err_context,
                note_item_array,
                instrument,
            )?);
        }
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
    let err_context = ErrorCallContext::from(&context);
    let instrument = eval_default_instrument(context.engine())?;
    let mut event_sequence = Vec::with_capacity(array.len());
    for item1_dyn in array {
        let item1_arr = unwrap_array(&err_context, item1_dyn)?;
        let mut note_events = Vec::with_capacity(item1_arr.len());
        if !item1_arr.is_empty()
            && (item1_arr[0].type_name() == "string" || item1_arr[0].is::<INT>())
        {
            // Vec<Vec<NOTE, VEL>>
            note_events.push(unwrap_note_event(&err_context, item1_arr, instrument)?);
        } else {
            // Vec<Vec<Vec<NOTE, VEL>>>
            for item2_dyn in item1_arr {
                let item2_arr = unwrap_array(&err_context, item2_dyn)?;
                note_events.push(unwrap_note_event(&err_context, item2_arr, instrument)?);
            }
        }
        event_sequence.push(note_events)
    }
    Ok(new_polyphonic_note_sequence_event(event_sequence))
}

fn notes_in_scale(
    context: NativeCallContext,
    string: ImmutableString,
) -> Result<Array, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    match scale::Scale::from_regex(string.as_str()) {
        Ok(scale) => Ok(scale
            .notes()
            .iter()
            .map(|n| Dynamic::from_int(Note::from(n) as u8 as INT))
            .collect::<Array>()),
        Err(_) => Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Invalid scale arg: '{}' in '{}'. Valid scale args are e.g. 'c major'",
                string,
                err_context.fn_name()
            )
            .into(),
            err_context.position(),
        )
        .into()),
    }
}

// ---------------------------------------------------------------------------------------------
// BeatTime

fn every_sixteenth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    every_nth_sixteenth(context, this, Dynamic::from_float(1.0))
}

fn every_nth_sixteenth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    sixteenth: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let step = unwrap_float(&err_context, sixteenth, "step")? as f32;
    Ok(this.every_nth_sixteenth(step))
}

fn every_eighth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    every_nth_eighth(context, this, Dynamic::from_float(1.0))
}

fn every_nth_eighth(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let step = unwrap_float(&err_context, beats, "step")? as f32;
    Ok(this.every_nth_eighth(step))
}

fn every_beat(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    every_nth_beat(context, this, Dynamic::from_float(1.0))
}

fn every_nth_beat(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    beats: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let step = unwrap_float(&err_context, beats, "step")? as f32;
    Ok(this.every_nth_beat(step))
}

fn every_bar(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    every_nth_bar(context, this, Dynamic::from_float(1.0))
}

fn every_nth_bar(
    context: NativeCallContext,
    this: &mut BeatTimeBase,
    bars: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let step = unwrap_float(&err_context, bars, "step")? as f32;
    Ok(this.every_nth_bar(step))
}

// ---------------------------------------------------------------------------------------------
// BeatTimeRhythm

fn with_pattern(
    context: NativeCallContext,
    this: &mut BeatTimeRhythm,
    pattern: Array,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let mut vec = Vec::with_capacity(pattern.len());
    for e in pattern {
        vec.push(unwrap_integer(&err_context, e, "array element")?)
    }
    Ok(this.with_pattern_vector(vec))
}

fn with_offset(
    context: NativeCallContext,
    this: &mut BeatTimeRhythm,
    offset: Dynamic,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let err_context = ErrorCallContext::from(&context);
    let offset = unwrap_float(&err_context, offset, "offset")? as f32;
    Ok(this.with_offset_in_step(offset))
}

fn trigger_fixed_event(this: &mut BeatTimeRhythm, event: FixedEventIter) -> BeatTimeRhythm {
    this.trigger(event)
}

fn trigger_custom_event(
    context: NativeCallContext,
    this: &mut BeatTimeRhythm,
    func: FnPtr,
) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
    let instrument = eval_default_instrument(context.engine())?;
    Ok(this.trigger(ScriptedEventIter::new(&context, func, instrument)?))
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        bindings::{eval_default_instrument, new_engine},
        event::{fixed::FixedEventIter, Event, InstrumentId},
        midi::Note,
        prelude::BeatTimeStep,
        rhythm::beat_time::BeatTimeRhythm,
        BeatTimeBase,
    };

    use super::{eval_default_beat_time, register};
    use rhai::{Dynamic, Engine};

    #[test]
    fn defaults() {
        // create a new engine and register bindings
        let mut engine = new_engine();
        register(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
            Some(InstrumentId::from(76)),
        );

        assert!(eval_default_beat_time(&engine).is_ok());
        let default_beat_time = eval_default_beat_time(&engine).unwrap();
        assert_eq!(default_beat_time.beats_per_min, 160.0);
        assert_eq!(default_beat_time.beats_per_bar, 6);
        assert_eq!(default_beat_time.samples_per_sec, 96000);

        assert!(eval_default_instrument(&engine).is_ok());
        assert_eq!(
            eval_default_instrument(&engine).unwrap(),
            Some(InstrumentId::from(76))
        );
    }

    #[test]
    fn extensions() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        register(
            &mut engine,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
            Some(InstrumentId::from(76)),
        );

        // Array::repeat
        assert!(engine.eval::<Dynamic>(r#"[1,2].repeat(-1)"#).is_err());
        let eval_result = engine.eval::<Dynamic>(r#"[1,2].repeat(2)"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let array = eval_result.unwrap().into_array().unwrap();
            assert!(
                array.len() == 4
                    && array[0].as_int() == Ok(1)
                    && array[1].as_int() == Ok(2)
                    && array[2].as_int() == Ok(1)
                    && array[3].as_int() == Ok(2)
            );
        }
    }

    #[test]
    fn note() {
        // create a new engine and register bindings
        let mut engine = new_engine();
        register(
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

        assert!(engine.eval::<Dynamic>(r#"note(["X#1", 0.5])"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note(["C#1", "0.5"])"#).is_err());
        assert!(engine
            .eval::<Dynamic>(r#"note(["C#1", 0.5, 1.0])"#)
            .is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note(["C#1", 0.5])"#);
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

        let eval_result = engine.eval::<Dynamic>(r#"note([0x3E, 0.5])"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let note_event = eval_result.unwrap().try_cast::<FixedEventIter>();
            assert!(
                if let Event::NoteEvents(notes) = &note_event.unwrap().events()[0] {
                    notes.len() == 1 && notes[0].note == Note::D4 && notes[0].velocity == 0.5
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

        // Notes in Scale
        assert!(engine
            .eval::<Dynamic>(r#"notes_in_scale("c wurst")"#)
            .is_err());
        assert_eq!(
            engine
                .eval::<Vec<rhai::Dynamic>>(r#"notes_in_scale("c major")"#)
                .unwrap()
                .iter()
                .map(|v| v.clone().cast::<rhai::INT>())
                .collect::<Vec<rhai::INT>>(),
            vec![60, 62, 64, 65, 67, 69, 71, 72]
        );
    }

    #[test]
    fn beat_time() {
        // create a new engine and register bindings
        let mut engine = new_engine();
        register(
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
