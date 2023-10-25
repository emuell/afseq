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
    event::{fixed::FixedEventIter, scripted::rhai::ScriptedEventIter},
    rhythm::{beat_time::BeatTimeRhythm, euclidean::euclidean},
};

use anyhow::anyhow;
use rhai::{packages::Package, plugin::*, Dynamic, Engine, EvalAltResult};
use rhai_rand::RandomPackage;
use rust_music_theory::{note::Notes, scale};

// ---------------------------------------------------------------------------------------------

pub(crate) mod unwrap;
use unwrap::*;

#[cfg(test)]
mod test;

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_engine() -> Engine {
    let mut engine = Engine::new();

    // Configure engine limits
    engine.set_max_expr_depths(1000, 1000);

    // load default packages
    engine.register_global_module(RandomPackage::new().as_shared_module());

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
                        .ok_or_else(|| anyhow!("Unexpected params array object"))?;
                    for item in array {
                        let object = item
                            .as_object()
                            .ok_or_else(|| anyhow!("Unexpected params array item"))?;
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
        Err(anyhow!("Unexpected meta data JSON").into())
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
    register_bindings(&mut engine, time_base, Some(instrument));

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
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
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
        Rc::new(RefCell::new(BeatTimeRhythm::new(
            time_base,
            BeatTimeStep::Beats(1.0),
        )))
    })
}

// ---------------------------------------------------------------------------------------------

/// Register afseq API bindings into the rhai engine.  
pub fn register_bindings(
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
    use rhai::*;

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
    use rhai::*;

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

    /// Create a note iterator, which endlessly emits a single fixed note.
    /// Note argument is a rhai object map with required "key" property and an
    /// optional "volume" property.
    ///
    /// # Example
    ///
    /// ```rhai
    /// note(#{key: 48, volume: 0.25})
    /// note(#{key: "C4"})
    /// ```
    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_map(
        context: NativeCallContext,
        map: rhai::Map,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        if let Some(note_event) = unwrap_note_event_from_map(&err_context, map, instrument)? {
            Ok(note_event.to_event())
        } else {
            Ok(new_empty_note_event())
        }
    }

    /// Create a note iterator, which endlessly emits a single fixed note.
    /// Note argument is a string with a required "key" and optional "volume".
    ///
    /// # Example
    ///
    /// ```rhai
    /// note("60 0.25")
    /// note("C4")
    /// note("---")
    /// ```
    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_string(
        context: NativeCallContext,
        s: ImmutableString,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        if is_empty_note_string(s.as_str()) {
            Ok(new_empty_note_event())
        } else {
            let err_context = ErrorCallContext::from(&context);
            let instrument = eval_default_instrument(context.engine())?;
            if let Some(note_event) =
                unwrap_note_event_from_string(&err_context, s.as_str(), instrument)?
            {
                Ok(note_event.to_event())
            } else {
                Ok(new_empty_note_event())
            }
        }
    }

    /// Create a note iterator, which endlessly emits a single fixed note.
    /// Note argument is an integer midi key value in range [0..128].
    ///
    /// # Example
    ///
    /// ```rhai
    /// note(60)
    /// ```
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
            1.0,
        ))
    }

    /// Create a note iterator, which endlessly emits a single fixed polyphonic note.
    /// Array items can be strings, integers, object maps or ().
    ///
    /// # Example
    ///
    /// ```rhai
    /// note([60, 62, 65]) // C Major Chord
    /// note(["C4", "E4", "G4"]  
    ///   .map(|n, i| #{key: n, volume: 0.3}))  // add volume 0.3 to all notes
    /// )
    /// note(["---",    "OFF",    "OFF"   ])
    /// ```
    #[rhai_fn(name = "note", return_raw)]
    pub fn note_from_array(
        context: NativeCallContext,
        array: Array,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        Ok(unwrap_note_events_from_array(&err_context, array, instrument)?.to_event())
    }

    /// Create a note iterator, which endlessly emits a sequence of monophonic or
    /// polyphonic notes.
    /// The sequence repeats from the beginning after the last note was emitted.
    /// Array items can be arrays, strings, integers, object maps or ().
    ///
    /// # Example
    ///
    /// ```rhai
    /// note_seq(["C4", "E4", "G4"]) // C Major Chord !arpeggio!
    /// note_seq([
    ///     ["A 3", "D 3", "F 3"], // Dm
    ///     ["B 3", "D 3", "G 3"], // G
    ///     ["C 3", "E 3", "A 3"], // C
    ///     ["---", "---", "---"], // continue last
    /// ])
    /// ```
    #[rhai_fn(name = "note_seq", return_raw)]
    pub fn note_seq_from_array(
        context: NativeCallContext,
        array: Array,
    ) -> Result<FixedEventIter, Box<EvalAltResult>> {
        let err_context = ErrorCallContext::from(&context);
        let instrument = eval_default_instrument(context.engine())?;
        let mut event_sequence = Vec::with_capacity(array.len());
        for item in array {
            event_sequence.push(unwrap_note_events_from_dynamic(
                &err_context,
                item,
                instrument,
            )?);
        }
        Ok(event_sequence.to_event_sequence())
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

    #[rhai_fn(name = "euclidean", return_raw)]
    pub fn euclidean_rhythm(
        context: NativeCallContext,
        pulses: INT,
        steps: INT,
    ) -> Result<Array, Box<EvalAltResult>> {
        euclidean_rhythm_with_offset(context, pulses, steps, 0)
    }

    #[rhai_fn(name = "euclidean", return_raw)]
    pub fn euclidean_rhythm_with_offset(
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
    use rhai::*;

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
    use rhai::*;

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
    use rhai::*;

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
    use rhai::*;

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
