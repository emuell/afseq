use crate::{bindings::rhai::*, prelude::*};

use ::rhai::{Dynamic, Engine, INT};

#[test]
fn defaults() {
    // create a new engine and register bindings
    let mut engine = new_engine();
    register_bindings(
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
    register_bindings(
        &mut engine,
        BeatTimeBase {
            beats_per_min: 160.0,
            beats_per_bar: 6,
            samples_per_sec: 96000,
        },
        Some(InstrumentId::from(76)),
    );

    assert!(crate::bindings::rhai::registered_functions(&engine).is_ok());
}

#[test]
fn extensions() {
    // create a new engine and register bindings
    let mut engine = Engine::new();
    register_bindings(
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
fn globals() {
    // create a new engine and register bindings
    let mut engine = new_engine();
    register_bindings(
        &mut engine,
        BeatTimeBase {
            beats_per_min: 120.0,
            beats_per_bar: 4,
            samples_per_sec: 44100,
        },
        None,
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
            .map(|v| v.clone().cast::<INT>())
            .collect::<Vec<INT>>(),
        vec![60, 62, 64, 65, 67, 69, 71, 72]
    );
}

#[test]
fn note() {
    // create a new engine and register bindings
    let mut engine = new_engine();
    register_bindings(
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

    // Note On (string)
    assert!(engine.eval::<Dynamic>(r#"note("X#1")"#).is_err());
    assert!(engine.eval::<Dynamic>(r#"note("0.5")"#).is_err());
    assert!(engine.eval::<Dynamic>(r#"note("C#1 -0.5")"#).is_err());
    assert!(engine.eval::<Dynamic>(r#"note("C#1", 0.5)"#).is_err());
    let eval_result = engine.eval::<Dynamic>(r#"note("C#1 0.5")"#).unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events()[0],
        Event::NoteEvents(vec![Some(new_note(None, "c#1", 0.5))])
    );

    // Note On (string array)
    assert!(engine.eval::<Dynamic>(r#"note(["X#1"])"#).is_err());
    let eval_result = engine.eval::<Dynamic>(r#"note(["C#1"])"#).unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events()[0],
        Event::NoteEvents(vec![Some(new_note(None, "c#1", 1.0))])
    );

    assert!(engine.eval::<Dynamic>(r#"note(["X#1 0.5"])"#).is_err());
    assert!(engine.eval::<Dynamic>(r#"note(["C#1 abc"])"#).is_err());
    let eval_result = engine
        .eval::<Dynamic>(r#"note(["C#1 0.5", "C5"])"#)
        .unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events()[0],
        Event::NoteEvents(vec![
            Some(new_note(None, "c#1", 0.5)),
            Some(new_note(None, "c5", 1.0))
        ])
    );

    // Note On (int)
    let eval_result = engine.eval::<Dynamic>(r#"note(0x3E)"#).unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events(),
        vec![Event::NoteEvents(vec![Some(new_note(None, "d4", 1.0))])]
    );

    // Note On (int array)
    let eval_result = engine.eval::<Dynamic>(r#"note([0x3E, 60])"#).unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events(),
        vec![Event::NoteEvents(vec![
            Some(new_note(None, "d4", 1.0)),
            Some(new_note(None, "c4", 1.0))
        ])]
    );

    // Note On (object map)
    assert!(engine.eval::<Dynamic>(r#"note(#{volume: 0.5})"#).is_err());
    assert!(engine
        .eval::<Dynamic>(r#"note(#{key: "xxx", volume: 0.5})"#)
        .is_err());
    assert!(engine
        .eval::<Dynamic>(r#"note(#{key: "C#1", volume: "abc"})"#)
        .is_err());
    assert!(engine
        .eval::<Dynamic>(r#"note(#{key: "C#1", volume: -1})"#)
        .is_err());
    let eval_result = engine
        .eval::<Dynamic>(r#"note(#{key: "G8", volume: 2})"#)
        .unwrap();
    let note_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_event.events()[0],
        Event::NoteEvents(vec![Some(new_note(None, "g8", 2.0)),])
    );

    // Note On (object map array)
    let eval_result = engine
        .eval::<Dynamic>(r#"note([#{key: "C#1", volume: 0.5}, #{key: "G2", volume: 0.75}, #{}])"#)
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
}

#[test]
fn note_sequence() {
    // create a new engine and register bindings
    let mut engine = Engine::new();
    let instrument = Some(InstrumentId::from(76));
    register_bindings(
        &mut engine,
        BeatTimeBase {
            beats_per_min: 160.0,
            beats_per_bar: 6,
            samples_per_sec: 96000,
        },
        instrument,
    );

    // Note Sequence
    let eval_result = engine
        .eval::<Dynamic>(r#"note_seq([["C#1 0.5"], ["---"], ["G_2"]])"#)
        .unwrap();
    let note_sequence_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        note_sequence_event.events(),
        vec![
            Event::NoteEvents(vec![Some(new_note(instrument, "c#1", 0.5))]),
            Event::NoteEvents(vec![None]),
            Event::NoteEvents(vec![Some(new_note(instrument, "g2", 1.0))])
        ]
    );

    let eval_result = engine
        .eval::<Dynamic>(
            r#"note_seq([
                    ["C#1", (), "G_2 0.75"], 
                    ["A#5 0.2", "---", #{key: "B_1", volume: 0.1}],
                ])"#,
        )
        .unwrap();
    let poly_note_sequence_event = eval_result.try_cast::<FixedEventIter>().unwrap();
    assert_eq!(
        poly_note_sequence_event.events(),
        vec![
            Event::NoteEvents(vec![
                Some(new_note(instrument, "c#1", 1.0)),
                None,
                Some(new_note(instrument, "g2", 0.75)),
            ]),
            Event::NoteEvents(vec![
                Some(new_note(instrument, "a#5", 0.2)),
                None,
                Some(new_note(instrument, "b1", 0.1))
            ])
        ]
    );
}

#[test]
fn beat_time() {
    // create a new engine and register bindings
    let mut engine = new_engine();
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

#[test]
fn second_time() {
    // create a new engine and register bindings
    let mut engine = new_engine();
    register_bindings(
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
