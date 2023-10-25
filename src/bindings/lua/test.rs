use crate::bindings::lua::*;

#[test]
fn extensions() {
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
    )
    .unwrap();

    // Pattern is present
    assert!(engine.load(r#"pattern.new()"#).eval::<LuaTable>().is_ok());
    // Fun is present
    assert!(engine
        .load(r#"fun.map(function(v) return v end, {1,2,3})"#)
        .eval::<LuaTable>()
        .is_ok());
}

#[test]
fn globals() {
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
    )
    .unwrap();
    // Notes in Scale
    assert!(engine
        .load(r#"notes_in_scale("c wurst")"#)
        .eval::<LuaValue>()
        .is_err());
    assert_eq!(
        engine
            .load(r#"notes_in_scale("c major")"#)
            .eval::<Vec<LuaValue>>()
            .unwrap()
            .iter()
            .map(|v| v.as_i32().unwrap())
            .collect::<Vec<i32>>(),
        vec![60, 62, 64, 65, 67, 69, 71, 72]
    );
}

#[test]
fn note() -> Result<(), Box<dyn std::error::Error>> {
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
    )?;

    // unwrap helper
    fn evaluate_chord_userdata(engine: &Lua, expression: &str) -> mlua::Result<ChordUserData> {
        Ok(engine
            .load(expression)
            .eval::<LuaValue>()?
            .as_userdata()
            .ok_or(mlua::Error::RuntimeError("No user data".to_string()))?
            .borrow::<ChordUserData>()?
            .clone())
    }

    // Empty Note
    let note_event = evaluate_chord_userdata(&engine, r#"chord("---")"#)?;
    assert_eq!(note_event.notes, vec![None]);

    // Note Off
    assert!(evaluate_chord_userdata(&engine, r#"chord("X#1")"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord("C#-2"#).is_err());
    let note_event = evaluate_chord_userdata(&engine, r#"chord("g#1")"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "g#1", 1.0))]);

    // Note On (string)
    assert!(evaluate_chord_userdata(&engine, r#"chord("X#1")"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord("0.5")"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord("C#1 -0.5")"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord("C#1", 0.5)"#).is_err());
    let note_event = evaluate_chord_userdata(&engine, r#"chord("C#1 0.5")"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "c#1", 0.5))]);

    // Note On (string array)
    assert!(evaluate_chord_userdata(&engine, r#"chord({"X#1"})"#).is_err());
    let note_event = evaluate_chord_userdata(&engine, r#"chord({"C#1"})"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "c#1", 1.0))]);

    assert!(evaluate_chord_userdata(&engine, r#"chord("X#1")"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord("C#1 abc")"#).is_err());
    let note_event = evaluate_chord_userdata(&engine, r#"chord({"C#1 0.5", "C5"})"#)?;
    assert_eq!(
        note_event.notes,
        vec![
            Some(new_note(None, "c#1", 0.5)),
            Some(new_note(None, "c5", 1.0))
        ]
    );

    // Note On (int)
    let note_event = evaluate_chord_userdata(&engine, r#"chord(0x3E)"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "d4", 1.0))]);

    // Note On (int array)
    let note_event = evaluate_chord_userdata(&engine, r#"chord({0x3E, 60})"#)?;
    assert_eq!(
        note_event.notes,
        vec![
            Some(new_note(None, "d4", 1.0)),
            Some(new_note(None, "c4", 1.0))
        ]
    );

    // Note On (table)
    assert!(evaluate_chord_userdata(&engine, r#"chord({volume = 0.5})"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord({key = "xxx", volume = 0.5})"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord({key = "C#1", volume = "abc"})"#).is_err());
    assert!(evaluate_chord_userdata(&engine, r#"chord({key = "C#1", volume = -1})"#).is_err());
    let note_event = evaluate_chord_userdata(&engine, r#"chord({key = "c8"})"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "c8", 1.0))]);
    let note_event = evaluate_chord_userdata(&engine, r#"chord({key = "G8", volume = 2})"#)?;
    assert_eq!(note_event.notes, vec![Some(new_note(None, "g8", 2.0))]);

    // Note On (object map array)
    let poly_note_event = evaluate_chord_userdata(
        &engine,
        r#"chord({{key = "C#1", volume = 0.5}, {key = "G2", volume = 0.75}, {}})"#,
    )?;
    assert_eq!(
        poly_note_event.notes,
        vec![
            Some(new_note(None, "c#1", 0.5)),
            Some(new_note(None, "g2", 0.75)),
            None
        ]
    );

    Ok(())
}

#[test]
fn sequence() -> Result<(), Box<dyn std::error::Error>> {
    // create a new engine and register bindings
    let mut engine = new_engine();
    let instrument = Some(InstrumentId::from(76));
    register_bindings(
        &mut engine,
        BeatTimeBase {
            beats_per_min: 120.0,
            beats_per_bar: 4,
            samples_per_sec: 44100,
        },
        instrument,
    )?;

    // unwrap helper
    fn evaluate_sequence_userdata(
        engine: &Lua,
        expression: &str,
    ) -> mlua::Result<SequenceUserData> {
        Ok(engine
            .load(expression)
            .eval::<LuaValue>()?
            .as_userdata()
            .ok_or(mlua::Error::RuntimeError("No user data".to_string()))?
            .borrow::<SequenceUserData>()?
            .clone())
    }

    // Note Sequence
    let note_sequence_event =
        evaluate_sequence_userdata(&engine, r#"sequence({"C#1 0.5"}, {"---"}, {"G_2"})"#)?;
    assert_eq!(
        note_sequence_event.notes,
        vec![
            vec![Some(new_note(instrument, "c#1", 0.5))],
            vec![None],
            vec![Some(new_note(instrument, "g2", 1.0))]
        ]
    );

    let poly_note_sequence_event = evaluate_sequence_userdata(
        &engine,
        r#"sequence(
                    {"C#1", "", "G_2 0.75"},
                    {"A#5 0.2", "---", {key = "B_1", volume = 0.1}}
                )"#,
    )?;
    assert_eq!(
        poly_note_sequence_event.notes,
        vec![
            vec![
                Some(new_note(instrument, "c#1", 1.0)),
                None,
                Some(new_note(instrument, "g2", 0.75)),
            ],
            vec![
                Some(new_note(instrument, "a#5", 0.2)),
                None,
                Some(new_note(instrument, "b1", 0.1))
            ]
        ]
    );

    Ok(())
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
    )
    .unwrap();

    // BeatTimeRhythm
    let beat_time_rhythm = engine
        .load(
            r#"
                Emitter {
                    unit = "beats",
                    resolution = 0.5,
                    offset = "2",
                    pattern = {1,0,1,0}
                }
            "#,
        )
        .eval::<LuaValue>()
        .unwrap();
    let beat_time_rhythm = beat_time_rhythm
        .as_userdata()
        .unwrap()
        .borrow::<BeatTimeRhythm>();
    assert!(beat_time_rhythm.is_ok());
    assert_eq!(
        beat_time_rhythm.as_ref().unwrap().step(),
        BeatTimeStep::Beats(0.5)
    );
    assert_eq!(
        beat_time_rhythm.as_ref().unwrap().offset(),
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
    )
    .unwrap();

    // SecondTimeRhythm
    let second_time_rhythm = engine
        .load(
            r#"
                Emitter {
                    unit = "seconds",
                    resolution = 2,
                    offset = 3,
                    pattern = {1,0,1,0}
                }
            "#,
        )
        .eval::<LuaValue>()
        .unwrap();

    let second_time_rhythm = second_time_rhythm
        .as_userdata()
        .unwrap()
        .borrow::<SecondTimeRhythm>();
    assert!(second_time_rhythm.is_ok());
    assert_eq!(second_time_rhythm.as_ref().unwrap().step(), 2.0);
    assert_eq!(second_time_rhythm.as_ref().unwrap().offset(), 3.0);
    assert_eq!(
        second_time_rhythm.unwrap().pattern(),
        vec![true, false, true, false]
    );
}
