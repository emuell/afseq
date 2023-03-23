//! Rhai script bindings for the entire crate.

use std::{
    cell::RefCell,
    env::temp_dir,
    fs::{remove_file, File},
    io::Write,
    path::PathBuf,
    rc::Rc,
};

use crate::prelude::*;
use crate::{
    event::{fixed::FixedEventIter, scripted::ScriptedEventIter},
    rhythm::{beat_time::BeatTimeRhythm, euclidian::euclidean},
};

use rhai::{
    packages::Package, plugin::*, Array, Dynamic, Engine, EvalAltResult, FnPtr, ImmutableString,
    NativeCallContext, FLOAT, INT,
};

use rhai_rand::RandomPackage;

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
    let rand = RandomPackage::new();
    rand.register_into_engine(&mut engine);

    engine
}

// -------------------------------------------------------------------------------------------------
pub struct FnMetaDataParam {
    pub name: Option<String>,
    pub type_: Option<String>,
}

pub struct FnMetaData {
    pub namespace: String,
    pub doc_comments: Vec<String>,
    pub signature: String,
    pub name: String,
    pub num_params: usize,
    pub params: Vec<FnMetaDataParam>,
    pub return_type: String,
}

pub fn registered_functions(
    engine: &Engine,
) -> Result<Vec<FnMetaData>, Box<dyn std::error::Error>> {
    let include_standard_packages = false;
    // dump metadata json: see https://rhai.rs/book/engine/metadata/export_to_json.html
    let string = engine.gen_fn_metadata_to_json(include_standard_packages)?;
    // deserialize from json, all other meta data access is private
    let value: serde_json::Value = serde_json::from_str(&string)?;
    let mut metadata = Vec::new();
    if let Some(array) = value["functions"].as_array() {
        for array_item in array.iter() {
            let get_string = move |name| {
                if let Some(value) = array_item.get(name) {
                    value.as_str().unwrap_or("").to_string()
                } else {
                    "".to_string()
                }
            };
            let get_string_list = move |name| {
                if let Some(value) = array_item.get(name) {
                    let mut strings = Vec::new();
                    for iter in value.as_array().unwrap_or(&Vec::new()) {
                        strings.push(iter.as_str().unwrap_or("").to_string())
                    }
                    strings
                } else {
                    Vec::new()
                }
            };
            let get_number = move |name| {
                if let Some(value) = array_item.get(name) {
                    value.as_u64().unwrap_or(0_u64) as usize
                } else {
                    0_usize
                }
            };
            let get_params = move || -> Result<Vec<FnMetaDataParam>, Box<dyn std::error::Error>> {
                if let Some(value) = array_item.get("params") {
                    let mut result: Vec<FnMetaDataParam> = Vec::new();
                    let array = value
                        .as_array()
                        .ok_or_else(|| string_error::new_err("Unexpected params array object"))?;
                    for item in array {
                        let object = item
                            .as_object()
                            .ok_or_else(|| string_error::new_err("Unexpected params array item"))?;
                        let mut param_name = None;
                        if let Some(name) = object.get("name") {
                            param_name = Some(name.as_str().unwrap_or("").to_string());
                        }
                        let mut param_type = None;
                        if let Some(type_) = object.get("type") {
                            param_type = Some(type_.as_str().unwrap_or("").to_string());
                        }
                        result.push(FnMetaDataParam {
                            name: param_name,
                            type_: param_type,
                        });
                    }
                    Ok(result)
                } else {
                    Ok(Vec::new())
                }
            };
            // only include public functions
            if get_string("access") == "public" && get_string("type") == "native" {
                metadata.push(FnMetaData {
                    namespace: get_string("namespace"),
                    doc_comments: get_string_list("docComments"),
                    signature: get_string("signature"),
                    name: get_string("name"),
                    return_type: get_string("returnType"),
                    num_params: get_number("numParams"),
                    params: get_params()?,
                });
            } else {
                debug_assert!(
                    false,
                    "Unexpected internal script function: {:?}",
                    array_item
                );
            }
        }
        Ok(metadata)
    } else {
        Err(string_error::static_err("Unexpected meta data JSON"))
    }
}

// -------------------------------------------------------------------------------------------------

// evaluate a script which creates and returns a Rhai rhythm to a Rust rhythm
pub fn new_rhythm_from_file(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // create a new engine
    let mut engine = new_engine();
    bindings::register(&mut engine, time_base, Some(instrument));

    // compile and evaluate script
    let ast = engine.compile_file(PathBuf::from(file_name))?;
    let result = engine.eval_ast::<Dynamic>(&ast)?;

    // hande script result
    if let Some(beat_time_rhythm) = result.clone().try_cast::<BeatTimeRhythm>() {
        Ok(Rc::new(RefCell::new(beat_time_rhythm)))
    } else if let Some(second_time_rhythm) = result.clone().try_cast::<SecondTimeRhythm>() {
        Ok(Rc::new(RefCell::new(second_time_rhythm)))
    } else {
        Err(EvalAltResult::ErrorMismatchDataType(
            "Rhythm".to_string(),
            result.type_name().to_string(),
            rhai::Position::new(1, 1),
        )
        .into())
    }
}

// evaluate a script which creates and returns a Rhai rhythm to a Rust rhythm,
// returning a fallback rhythm on errors
pub fn new_rhythm_from_file_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    file_name: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_file(instrument, time_base, file_name).unwrap_or_else(|err| {
        log::warn!("Script '{}' failed to compile: {}", file_name, err);
        Rc::new(RefCell::new(BeatTimeRhythm::new(time_base, BeatTimeStep::Beats(1.0))))
    })
}

// evaluate an expression which creates and returns a Rhai rhythm to a Rust rhythm
pub fn new_rhythm_from_string(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    script: &str,
) -> Result<Rc<RefCell<dyn Rhythm>>, Box<dyn std::error::Error>> {
    // HACK: Need to write the string to a file, so ScriptedEventIter can resolve functions
    let mut temp_file_name = temp_dir();
    temp_file_name.push("afseq/");
    std::fs::create_dir_all(temp_file_name.clone())?;
    temp_file_name.push(format!("{}.rhai", uuid::Uuid::new_v4()));

    let result = {
        let file = &mut File::create(temp_file_name.clone())?;
        file.write_all(script.as_bytes())?;
        new_rhythm_from_file(instrument, time_base, &temp_file_name.to_string_lossy())
    };
    remove_file(temp_file_name)?;
    result
}

// evaluate an expression which creates and returns a Rhai rhythm to a Rust rhythm,
// returning a fallback rhythm on errors
pub fn new_rhythm_from_string_with_fallback(
    instrument: InstrumentId,
    time_base: BeatTimeBase,
    expression: &str,
    expression_identifier: &str,
) -> Rc<RefCell<dyn Rhythm>> {
    new_rhythm_from_string(instrument, time_base, expression).unwrap_or_else(|err| {
        log::warn!(
            "Script '{}' failed to compile: {}",
            expression_identifier,
            err
        );
        Rc::new(RefCell::new(BeatTimeRhythm::new(time_base, BeatTimeStep::Beats(1.0))))
    })
}

// ---------------------------------------------------------------------------------------------

/// Register afseq API bindings into the rhai engine.  
pub fn register(
    engine: &mut Engine,
    default_time_base: BeatTimeBase,
    default_instrument: Option<InstrumentId>,
) {
    // Defaults
    let mut defaults = Module::new();
    defaults.set_var("DEFAULT_INSTRUMENT", Dynamic::from(default_instrument));
    defaults.set_var("DEFAULT_BEAT_TIME", Dynamic::from(default_time_base));
    engine.register_global_module(defaults.into());

    // Array Extensions
    let array = exported_module!(array_module);
    engine.register_global_module(array.into());

    // Globals
    let globals = exported_module!(globals_module);
    engine.register_global_module(globals.into());

    // TimeBase
    let beat_time = exported_module!(beat_time_module);
    engine.register_global_module(beat_time.into());
    let second_time = exported_module!(second_time_module);
    engine.register_global_module(second_time.into());

    // Rhythm
    let beat_time_rhythm = exported_module!(beat_time_rhythm_module);
    engine.register_global_module(beat_time_rhythm.into());
    let second_time_rhythm = exported_module!(second_time_rhythm_module);
    engine.register_global_module(second_time_rhythm.into());
}

// ---------------------------------------------------------------------------------------------

fn eval_default_instrument(engine: &Engine) -> Result<Option<InstrumentId>, Box<EvalAltResult>> {
    engine.eval::<Option<InstrumentId>>("DEFAULT_INSTRUMENT")
}

fn eval_default_beat_time(engine: &Engine) -> Result<BeatTimeBase, Box<EvalAltResult>> {
    engine.eval::<BeatTimeBase>("DEFAULT_BEAT_TIME")
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod array_module {
    use super::*;

    /// Repeats/duplicates an array n times.
    /// @param count: how many times the array should be repeated
    /// @return The repeated, duplicated array.
    #[rhai_fn(name = "repeat", return_raw)]
    pub fn repeat(
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

    /// Reverse entries in an array.
    /// @return A copy of the reversed array.
    #[rhai_fn(name = "reverse")]
    pub fn reverse(this: Array) -> Array {
        this.iter().rev().cloned().collect::<Array>()
    }

    /// Rotate entries in an array.
    /// param offset positive or negative shifting offset.
    /// @return A copy of the rotated array.
    #[rhai_fn(name = "rotate")]
    pub fn rotate(this: Array, offset: INT) -> Array {
        if this.is_empty() {
            return Array::new();
        }
        let mut ret = this;
        let size = ret.len();
        match offset {
            n if n > 0 => ret.rotate_right((n as usize) % size),
            n if n < 0 => ret.rotate_left((-n as usize) % size),
            _ => (),
        }
        ret
    }
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod globals_module {
    use super::*;

    #[rhai_fn(name = "beat_time", return_raw)]
    pub fn beat_time_default(
        context: NativeCallContext,
    ) -> Result<BeatTimeBase, Box<EvalAltResult>> {
        eval_default_beat_time(context.engine())
    }

    #[rhai_fn(name = "beat_time", return_raw)]
    pub fn beat_time(
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

    #[rhai_fn(name = "second_time", return_raw)]
    pub fn second_time(context: NativeCallContext) -> Result<SecondTimeBase, Box<EvalAltResult>> {
        let default_beat_time = eval_default_beat_time(context.engine())?;
        Ok(SecondTimeBase {
            samples_per_sec: default_beat_time.samples_per_sec,
        })
    }

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_dynamic(
        context: NativeCallContext,
        d: Dynamic,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        if d.is::<()>() {
            Ok(new_empty_note_event())
        } else if d.is::<ImmutableString>() || d.is::<String>() {
            Ok(note_from_string(context, d.cast())?)
        } else if d.is::<INT>() {
            Ok(note_from_number(context, d.cast())?)
        } else {
            Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': Argument is neither (), number, or a string, but is a '{}'.",
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
        }
    }

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_string(
        context: NativeCallContext,
        note: ImmutableString,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        if is_empty_note_string(note.as_str()) {
            Ok(new_empty_note_event())
        } else {
            Ok(new_note_event(
                instrument,
                unwrap_note_from_string(&err_context, note.as_str())?,
                1.0_f32,
            ))
        }
    }

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_number(
        context: NativeCallContext,
        note: INT,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        Ok(new_note_event(
            instrument,
            unwrap_note_from_int(&err_context, note)?,
            1.0_f32,
        ))
    }

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_dynamic_with_velocity(
        context: NativeCallContext,
        d: Dynamic,
        velocity: FLOAT,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        if d.is::<()>() {
            Ok(new_empty_note_event())
        } else if d.is::<ImmutableString>() || d.is::<String>() {
            Ok(note_from_string_with_velocity(context, d.cast(), velocity)?)
        } else if d.is::<INT>() {
            Ok(note_from_number_with_velocity(context, d.cast(), velocity)?)
        } else {
            Err(EvalAltResult::ErrorInModule(
            "bindings".to_string(),
            format!(
                "Failed to parse note in function '{}': Argument is neither (), number, or a string, but is a '{}'.",
                context.fn_name(),
                d.type_name()
            )
            .into(),
            context.position(),
        )
        .into())
        }
    }

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_string_with_velocity(
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

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_number_with_velocity(
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

    #[rhai_fn(name = "note", return_raw)]
    pub fn note_vec(
        context: NativeCallContext,
        array: Array,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        // NB: array arg may be a:
        // [NOTE, VEL] -> single note
        // [[NOTE, VEL], ..] -> poly notes
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        let mut sequence = Vec::with_capacity(array.len());
        if array.is_empty() {
            // []
            sequence.push(None);
        } else if array[0].type_name() == "string" || array[0].is::<INT>() || array[0].is::<()>() {
            // [NOTE, VEL]
            if is_empty_note_value(&array[0]) {
                sequence.push(None);
            } else {
                sequence.push(Some(unwrap_note_event(&err_context, array, instrument)?));
            }
        } else {
            // [[NOTE, VEL], ..]
            for item in array {
                if item.is::<()>() {
                    sequence.push(None);
                } else {
                    let note_item_array = unwrap_array(&err_context, item)?;
                    if note_item_array.is_empty() || is_empty_note_value(&note_item_array[0]) {
                        sequence.push(None);
                    } else {
                        sequence.push(Some(unwrap_note_event(
                            &err_context,
                            note_item_array,
                            instrument,
                        )?));
                    }
                }
            }
        }
        Ok(new_polyphonic_note_event(sequence))
    }

    #[rhai_fn(name = "note_seq", return_raw)]
    pub fn note_vec_seq(
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
            if item1_dyn.is::<()>() {
                event_sequence.push(vec![None]);
            } else {
                let item1_arr = unwrap_array(&err_context, item1_dyn)?;
                let mut note_events = Vec::with_capacity(item1_arr.len());
                if item1_arr.is_empty() {
                    // Vec<()>
                    note_events.push(None);
                } else if item1_arr[0].type_name() == "string" || item1_arr[0].is::<INT>() {
                    // Vec<Vec<NOTE, VEL>>
                    if item1_arr.is_empty() || is_empty_note_value(&item1_arr[0]) {
                        note_events.push(None);
                    } else {
                        note_events.push(Some(unwrap_note_event(
                            &err_context,
                            item1_arr,
                            instrument,
                        )?));
                    }
                } else {
                    // Vec<Vec<Vec<NOTE, VEL>>>
                    for item2_dyn in item1_arr {
                        if item2_dyn.is::<()>() {
                            note_events.push(None);
                        } else {
                            let item2_arr = unwrap_array(&err_context, item2_dyn)?;
                            if item2_arr.is_empty() || is_empty_note_value(&item2_arr[0]) {
                                note_events.push(None);
                            } else {
                                note_events.push(Some(unwrap_note_event(
                                    &err_context,
                                    item2_arr,
                                    instrument,
                                )?));
                            }
                        }
                    }
                }
                event_sequence.push(note_events)
            }
        }
        Ok(new_polyphonic_note_sequence_event(event_sequence))
    }

    #[rhai_fn(name = "notes_in_scale", return_raw)]
    pub fn notes_in_scale(
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

    #[rhai_fn(name = "euclidian", return_raw)]
    pub fn euclidian_rhythm(
        context: NativeCallContext,
        pulses: INT,
        steps: INT,
    ) -> Result<Array, Box<EvalAltResult>> {
        euclidian_rhythm_with_offset(context, pulses, steps, 0)
    }

    #[rhai_fn(name = "euclidian", return_raw)]
    pub fn euclidian_rhythm_with_offset(
        context: NativeCallContext,
        pulses: INT,
        steps: INT,
        offset: INT,
    ) -> Result<Array, Box<EvalAltResult>> {
        if pulses <= 0 || steps <= 0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid arguments in fn '{}': 'pulse' (is {}) and 'step' (is {}) must be > 0'",
                    context.fn_name(),
                    pulses,
                    steps
                )
                .into(),
                context.position(),
            )
            .into())
        } else if pulses > steps {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid arguments in fn '{}': 'pulse' (is {}) must be <= 'step' (is {})",
                    context.fn_name(),
                    pulses,
                    steps
                )
                .into(),
                context.position(),
            )
            .into())
        } else {
            let pattern = euclidean(pulses as u32, steps as u32, offset as i32);
            Ok(pattern
                .iter()
                .map(|v| {
                    if *v {
                        Dynamic::from(1 as INT)
                    } else {
                        Dynamic::from(0 as INT)
                    }
                })
                .collect::<Array>())
        }
    }
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod beat_time_module {
    use super::*;

    #[rhai_fn(name = "every_sixteenth", return_raw)]
    pub fn every_sixteenth(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        every_nth_sixteenth(context, this, Dynamic::from_float(1.0))
    }

    #[rhai_fn(name = "every_nth_sixteenth", return_raw)]
    pub fn every_nth_sixteenth(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
        sixteenth: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let step = unwrap_float(&err_context, sixteenth, "step")? as f32;
        if step <= 0.0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid step arg: '{}' in '{}'. step must be > 0",
                    step,
                    err_context.fn_name()
                )
                .into(),
                err_context.position(),
            )
            .into())
        } else {
            Ok(this.every_nth_sixteenth(step))
        }
    }

    #[rhai_fn(name = "every_eighth", return_raw)]
    pub fn every_eighth(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        every_nth_eighth(context, this, Dynamic::from_float(1.0))
    }

    #[rhai_fn(name = "every_nth_eighth", return_raw)]
    pub fn every_nth_eighth(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
        beats: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let step = unwrap_float(&err_context, beats, "step")? as f32;
        if step <= 0.0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid step arg: '{}' in '{}'. step must be > 0",
                    step,
                    err_context.fn_name()
                )
                .into(),
                err_context.position(),
            )
            .into())
        } else {
            Ok(this.every_nth_eighth(step))
        }
    }

    #[rhai_fn(name = "every_beat", return_raw)]
    pub fn every_beat(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        every_nth_beat(context, this, Dynamic::from_float(1.0))
    }

    #[rhai_fn(name = "every_nth_beat", return_raw)]
    pub fn every_nth_beat(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
        beats: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let step = unwrap_float(&err_context, beats, "step")? as f32;
        if step <= 0.0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid step arg: '{}' in '{}'. step must be > 0",
                    step,
                    err_context.fn_name()
                )
                .into(),
                err_context.position(),
            )
            .into())
        } else {
            Ok(this.every_nth_beat(step))
        }
    }

    #[rhai_fn(name = "every_bar", return_raw)]
    pub fn every_bar(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        every_nth_bar(context, this, Dynamic::from_float(1.0))
    }

    #[rhai_fn(name = "every_nth_bar", return_raw)]
    pub fn every_nth_bar(
        context: NativeCallContext,
        this: &mut BeatTimeBase,
        bars: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let step = unwrap_float(&err_context, bars, "step")? as f32;
        if step <= 0.0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid step arg: '{}' in '{}'. step must be > 0",
                    step,
                    err_context.fn_name()
                )
                .into(),
                err_context.position(),
            )
            .into())
        } else {
            Ok(this.every_nth_bar(step))
        }
    }
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod second_time_module {
    use super::*;

    #[rhai_fn(name = "every_nth_second", return_raw)]
    pub fn every_nth_second(
        context: NativeCallContext,
        this: &mut SecondTimeBase,
        seconds: Dynamic,
    ) -> Result<SecondTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let step = unwrap_float(&err_context, seconds, "step")?;
        if step <= 0.0 {
            Err(EvalAltResult::ErrorInModule(
                "bindings".to_string(),
                format!(
                    "Invalid seconds arg: '{}' in '{}'. seconds must be > 0",
                    step,
                    err_context.fn_name()
                )
                .into(),
                err_context.position(),
            )
            .into())
        } else {
            Ok(this.every_nth_seconds(step))
        }
    }
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod beat_time_rhythm_module {
    use super::*;

    #[rhai_fn(name = "with_pattern", return_raw)]
    pub fn with_pattern(
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

    #[rhai_fn(name = "with_offset", return_raw)]
    pub fn with_offset(
        context: NativeCallContext,
        this: &mut BeatTimeRhythm,
        offset: Dynamic,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let offset = unwrap_float(&err_context, offset, "offset")? as f32;
        Ok(this.with_offset_in_step(offset))
    }

    #[rhai_fn(name = "trigger")]
    pub fn trigger_fixed_event(this: &mut BeatTimeRhythm, event: FixedEventIter) -> BeatTimeRhythm {
        this.trigger(event)
    }

    #[rhai_fn(name = "trigger", return_raw)]
    pub fn trigger_custom_event(
        context: NativeCallContext,
        this: &mut BeatTimeRhythm,
        func: FnPtr,
    ) -> Result<BeatTimeRhythm, Box<EvalAltResult>> {
        let instrument = eval_default_instrument(context.engine())?;
        Ok(this.trigger(ScriptedEventIter::new(&context, func, instrument)?))
    }
}

// ---------------------------------------------------------------------------------------------

#[export_module]
mod second_time_rhythm_module {
    use super::*;

    #[rhai_fn(name = "with_pattern", return_raw)]
    pub fn with_pattern(
        context: NativeCallContext,
        this: &mut SecondTimeRhythm,
        pattern: Array,
    ) -> Result<SecondTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let mut vec = Vec::with_capacity(pattern.len());
        for e in pattern {
            vec.push(unwrap_integer(&err_context, e, "array element")?)
        }
        Ok(this.with_pattern_vector(vec))
    }

    #[rhai_fn(name = "with_offset", return_raw)]
    pub fn with_offset(
        context: NativeCallContext,
        this: &mut SecondTimeRhythm,
        offset: Dynamic,
    ) -> Result<SecondTimeRhythm, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let offset = unwrap_float(&err_context, offset, "offset")?;
        Ok(this.with_offset(offset))
    }

    #[rhai_fn(name = "trigger")]
    pub fn trigger_fixed_event(
        this: &mut SecondTimeRhythm,
        event: FixedEventIter,
    ) -> SecondTimeRhythm {
        this.trigger(event)
    }

    #[rhai_fn(name = "trigger", return_raw)]
    pub fn trigger_custom_event(
        context: NativeCallContext,
        this: &mut SecondTimeRhythm,
        func: FnPtr,
    ) -> Result<SecondTimeRhythm, Box<EvalAltResult>> {
        let instrument = eval_default_instrument(context.engine())?;
        Ok(this.trigger(ScriptedEventIter::new(&context, func, instrument)?))
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        bindings::{eval_default_beat_time, eval_default_instrument, new_engine, register},
        event::{fixed::FixedEventIter, new_note, Event, InstrumentId},
        rhythm::{beat_time::BeatTimeRhythm, second_time::SecondTimeRhythm},
        time::BeatTimeStep,
        BeatTimeBase, SecondTimeBase,
    };

    use rhai::{Dynamic, Engine, INT};

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
    fn registered_functions() {
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

        assert!(super::registered_functions(&engine).is_ok());
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
            let array = eval_result
                .unwrap()
                .into_array()
                .unwrap()
                .iter()
                .map(|f| f.as_int().unwrap())
                .collect::<Vec<INT>>();
            assert_eq!(array, vec![1, 2, 1, 2]);
        }

        // Array::reverse
        assert!(engine.eval::<Dynamic>(r#"[].reverse()"#).is_ok());
        let eval_result = engine.eval::<Dynamic>(r#"[1,2].reverse()"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let array = eval_result
                .unwrap()
                .into_array()
                .unwrap()
                .iter()
                .map(|f| f.as_int().unwrap())
                .collect::<Vec<INT>>();
            assert_eq!(array, vec![2, 1]);
        }

        // Array::rotate
        assert!(engine.eval::<Dynamic>(r#"[1,2,3].rotate(0)"#).is_ok());
        let eval_result = engine.eval::<Dynamic>(r#"[1,2,3].rotate(1)"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let array = eval_result
                .unwrap()
                .into_array()
                .unwrap()
                .iter()
                .map(|f| f.as_int().unwrap())
                .collect::<Vec<INT>>();
            assert_eq!(array, vec![3, 1, 2]);
        }
        let eval_result = engine.eval::<Dynamic>(r#"[1,2,3].rotate(-1)"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let array = eval_result
                .unwrap()
                .into_array()
                .unwrap()
                .iter()
                .map(|f| f.as_int().unwrap())
                .collect::<Vec<INT>>();
            assert_eq!(array, vec![2, 3, 1]);
        }
        let eval_result = engine.eval::<Dynamic>(r#"[1,2,3].rotate(3)"#);
        if let Err(err) = eval_result {
            panic!("{}", err);
        } else {
            let array = eval_result
                .unwrap()
                .into_array()
                .unwrap()
                .iter()
                .map(|f| f.as_int().unwrap())
                .collect::<Vec<INT>>();
            assert_eq!(array, vec![1, 2, 3]);
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

        // Empty Note
        let eval_result = engine.eval::<Dynamic>(r#"note("---")"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(note_event.events()[0], Event::NoteEvents(vec![None]));

        // Note Off
        assert!(engine.eval::<Dynamic>(r#"note("X#1")"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note("C#-2"#).is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note("g#1")"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_event.events()[0],
            Event::NoteEvents(vec![Some(new_note(None, "g#1", 1.0))])
        );

        // Note
        assert!(engine.eval::<Dynamic>(r#"note("X#1", 0.5)"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note("C#1", "0.5")"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note("C#1", 0.5, 1.0)"#).is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note("C#1", 0.5)"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_event.events()[0],
            Event::NoteEvents(vec![Some(new_note(None, "c#1", 0.5))])
        );

        assert!(engine.eval::<Dynamic>(r#"note(["X#1"])"#).is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note(["C#1"])"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_event.events()[0],
            Event::NoteEvents(vec![Some(new_note(None, "c#1", 1.0))])
        );

        assert!(engine.eval::<Dynamic>(r#"note(["X#1", 0.5])"#).is_err());
        assert!(engine.eval::<Dynamic>(r#"note(["C#1", "0.5"])"#).is_err());
        assert!(engine
            .eval::<Dynamic>(r#"note(["C#1", 0.5, 1.0])"#)
            .is_err());
        let eval_result = engine.eval::<Dynamic>(r#"note(["C#1", 0.5])"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_event.events()[0],
            Event::NoteEvents(vec![Some(new_note(None, "c#1", 0.5))])
        );

        let eval_result = engine.eval::<Dynamic>(r#"note([0x3E, 0.5])"#).unwrap();
        let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_event.events(),
            vec![Event::NoteEvents(vec![Some(new_note(None, "d4", 0.5))])]
        );

        assert!(engine
            .eval::<Dynamic>(r#"note([["Note", 0.5, 1.0]])"#)
            .is_err());
        assert!(engine
            .eval::<Dynamic>(r#"note([["C#1", 0.5, 1.0]])"#)
            .is_err());
        assert!(engine.eval::<Dynamic>(r#"note([["C#1", "0.5"]])"#).is_err());
        let eval_result = engine
            .eval::<Dynamic>(r#"note([["C#1", 0.5], ["G2", 0.75], []])"#)
            .unwrap();
        let poly_note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            poly_note_event.events()[0],
            Event::NoteEvents(vec![
                Some(new_note(None, "c#1", 0.5)),
                Some(new_note(None, "g2", 0.75)),
                None
            ])
        );

        // NoteSequence
        let eval_result = engine
            .eval::<Dynamic>(r#"note_seq([["C#1", 0.5], ["---"], ["G_2"]])"#)
            .unwrap();
        let note_sequence_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            note_sequence_event.events(),
            vec![
                Event::NoteEvents(vec![Some(new_note(None, "c#1", 0.5))]),
                Event::NoteEvents(vec![None]),
                Event::NoteEvents(vec![Some(new_note(None, "g2", 1.0))])
            ]
        );

        let eval_result = engine
            .eval::<Dynamic>(
                r#"note_seq([
                     [["C#1"], (), ["G_2", 0.75]], 
                     [["A#5", 0.2], ["---"], ["B_1", 0.1]]
                   ])"#,
            )
            .unwrap();
        let poly_note_sequence_event = eval_result.try_cast::<FixedEventIter>().unwrap();
        assert_eq!(
            poly_note_sequence_event.events(),
            vec![
                Event::NoteEvents(vec![
                    Some(new_note(None, "c#1", 1.0)),
                    None,
                    Some(new_note(None, "g2", 0.75)),
                ]),
                Event::NoteEvents(vec![
                    Some(new_note(None, "a#5", 0.2)),
                    None,
                    Some(new_note(None, "b1", 0.1))
                ])
            ]
        );

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

    #[test]
    fn second_time() {
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

        // SecondTime
        assert!(engine
            .eval::<Dynamic>("second_time()",)
            .unwrap()
            .try_cast::<SecondTimeBase>()
            .is_some());

        // SecondTimeRhythm
        let second_time_rhythm = engine
            .eval::<Dynamic>(
                r#"
                  second_time()
                    .every_nth_second(2)
                    .with_offset(3)
                    .with_pattern([1,0,1,0]);
                "#,
            )
            .unwrap()
            .try_cast::<SecondTimeRhythm>();
        assert!(second_time_rhythm.is_some());
        assert_eq!(second_time_rhythm.clone().unwrap().step(), 2.0);
        assert_eq!(second_time_rhythm.clone().unwrap().offset(), 3.0);
        assert_eq!(
            second_time_rhythm.unwrap().pattern(),
            vec![true, false, true, false]
        );
    }
}
