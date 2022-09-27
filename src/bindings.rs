use std::sync::atomic::{AtomicU32, AtomicUsize};

use crate::prelude::*;
use crate::{
    event::fixed::FixedEventIter, event::sequence::EventIterSequence,
    rhythm::beat_time::BeatTimeRhythm, BeatTimeBase,
};

use rhai::{Array, Dynamic, Engine, EvalAltResult, ImmutableString, FLOAT, INT};

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

fn note(s: ImmutableString, velocity: FLOAT) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let instrument = INSTRUMENT_ID.load(std::sync::atomic::Ordering::Relaxed);
    let instrument = if instrument == usize::MAX {
        None
    } else {
        Some(instrument)
    };
    Ok(new_note_event(
        instrument,
        Note::try_from(s.as_str())?,
        velocity as f32,
    ))
}

fn note_vec(array: Array) -> Result<FixedEventIter, Box<EvalAltResult>> {
    let instrument = INSTRUMENT_ID.load(std::sync::atomic::Ordering::Relaxed);
    let instrument = if instrument == usize::MAX {
        None
    } else {
        Some(instrument)
    };
    let mut sequence = Vec::with_capacity(array.len());
    for item in array {
        let note_item_array = item.into_array()?;
        if note_item_array.len() != 2 {
            return Err(format!(
                "Expected 2 items in note array ['note', 'velocity'], got '{}' items",
                note_item_array.len()
            )
            .into());
        }
        let note = Note::try_from(note_item_array[0].clone().into_string()?.as_str())?;
        let velocity = unwrap_float(note_item_array[1].clone(), "velocity", "seq_vec")? as f32;
        sequence.push((instrument, note, velocity));
    }
    Ok(new_polyphonic_note_event(sequence))
}

fn note_vec_seq(array: Array) -> Result<EventIterSequence, Box<EvalAltResult>> {
    // NB: array arg may be a:
    // [[NOTE, VEL], ..] -> sequence of single notes
    // [[[NOTE, VEL], ..], [[NOTE, VEL]]] -> sequence of poly notes
    let instrument = INSTRUMENT_ID.load(std::sync::atomic::Ordering::Relaxed);
    let instrument = if instrument == usize::MAX {
        None
    } else {
        Some(instrument)
    };
    let mut event_sequence = Vec::with_capacity(array.len());
    for item1_dyn in array {
        let item1_array_result = item1_dyn.into_array();
        if let Err(other_type) = item1_array_result {
            return Err(format!("Expected 'array', got '{}'", other_type).into());
        }
        let item1_arr = item1_array_result.unwrap();
        let mut note_events = Vec::with_capacity(item1_arr.len());
        if !item1_arr.is_empty() && item1_arr[0].type_name() == "string" {
            // Vec<Vec<NOTE, VEL>>
            if item1_arr.len() != 2 {
                return Err(format!(
                    "Expected 2 items in note array ['note', 'velocity'], got '{}' items",
                    item1_arr.len()
                )
                .into());
            }
            let note = Note::try_from(item1_arr[0].clone().into_string()?.as_str())?;
            let velocity = unwrap_float(item1_arr[1].clone(), "velocity", "seq_vec")? as f32;
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
                let note = Note::try_from(item2_arr[0].clone().into_string()?.as_str())?;
                let velocity = unwrap_float(item2_arr[1].clone(), "velocity", "seq_vec")? as f32;
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
    pattern: Array,
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
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer as f64)
    } else {
        Err(format!(
            "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
            arg_name,
            func_name,
            d.type_name()
        )
        .as_str()
        .into())
    }
}

fn unwrap_integer(d: Dynamic, arg_name: &str, func_name: &str) -> Result<INT, Box<EvalAltResult>> {
    if let Some(float) = d.clone().try_cast::<FLOAT>() {
        Ok(float as INT)
    } else if let Some(integer) = d.clone().try_cast::<INT>() {
        Ok(integer)
    } else {
        Err(format!(
            "Invalid arg: '{}' in '{}' must be a number value, but is a '{}'",
            arg_name,
            func_name,
            d.type_name()
        )
        .as_str()
        .into())
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use crate::{
        event::{fixed::FixedEventIter, sequence::EventIterSequence, Event, Note},
        prelude::BeatTimeStep,
        rhythm::beat_time::BeatTimeRhythm,
        BeatTimeBase,
    };

    use super::{register_bindings, set_global_binding_state};
    use rhai::{Dynamic, Engine};

    #[test]
    fn note() {
        // create a new engine and register bindings
        let mut engine = Engine::new();
        set_global_binding_state(4800, None);
        register_bindings(&mut engine); // NoteEvent

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
                if let Event::NoteEvents(notes) = note_event.unwrap().event() {
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
            let note_event = poly_note_event.unwrap().event();
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
            let note_sequence_event = eval_result.unwrap().try_cast::<EventIterSequence>();
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
            .try_cast::<EventIterSequence>();
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
