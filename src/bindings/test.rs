use crate::bindings::*;

// --------------------------------------------------------------------------------------------------
// unwrap helpers

fn evaluate_note_userdata(engine: &Lua, expression: &str) -> mlua::Result<NoteUserData> {
    Ok(engine
        .load(expression)
        .eval::<LuaValue>()?
        .as_userdata()
        .ok_or(mlua::Error::RuntimeError("No user data".to_string()))?
        .borrow::<NoteUserData>()?
        .clone())
}

fn evaluate_sequence_userdata(engine: &Lua, expression: &str) -> mlua::Result<SequenceUserData> {
    Ok(engine
        .load(expression)
        .eval::<LuaValue>()?
        .as_userdata()
        .ok_or(mlua::Error::RuntimeError("No user data".to_string()))?
        .borrow::<SequenceUserData>()?
        .clone())
}

// --------------------------------------------------------------------------------------------------

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

    // Empty Note
    let note_event = evaluate_note_userdata(&engine, r#"note("---")"#)?;
    assert_eq!(note_event.notes, vec![None]);

    // Note Off
    assert!(evaluate_note_userdata(&engine, r#"note("off")"#).is_ok());
    let note_event = evaluate_note_userdata(&engine, r#"note("OFF")"#)?;
    assert_eq!(note_event.notes, vec![new_note((None, Note::OFF))]);

    // Note string
    assert!(evaluate_note_userdata(&engine, r#"note("X#1")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("0.5")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 -0.5")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1", 0.5)"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 ..")"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 .. -2.0")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 ..  -1.0")"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 .. ..")"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 .. .. -1.0")"#).is_err());
    let note_event = evaluate_note_userdata(&engine, r#"note("C#1 0.5 0.1 0.2")"#)?;
    assert_eq!(
        note_event.notes,
        vec![Some(NoteEvent {
            instrument: None,
            note: Note::Cs1,
            volume: 0.5,
            panning: 0.1,
            delay: 0.2
        })]
    );
    let note_event = evaluate_note_userdata(&engine, r#"note("C#1 .. .. 0.2")"#)?;
    assert_eq!(
        note_event.notes,
        vec![Some(NoteEvent {
            instrument: None,
            note: Note::Cs1,
            volume: 1.0,
            panning: 0.0,
            delay: 0.2
        })]
    );

    // Note string array
    assert!(evaluate_note_userdata(&engine, r#"note({"X#1"})"#).is_err());
    let note_event = evaluate_note_userdata(&engine, r#"note({"C#1"})"#)?;
    assert_eq!(note_event.notes, vec![new_note((None, "c#1"))]);

    assert!(evaluate_note_userdata(&engine, r#"note("X#1")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("C#1 abc")"#).is_err());
    let note_event = evaluate_note_userdata(&engine, r#"note({"C#1 0.5", "C5"})"#)?;
    assert_eq!(
        note_event.notes,
        vec![new_note((None, "c#1", 0.5)), new_note((None, "c5", 1.0))]
    );

    // Note int
    let note_event = evaluate_note_userdata(&engine, r#"note(0x32)"#)?;
    assert_eq!(note_event.notes, vec![new_note((None, "d4"))]);

    // Note int array
    let note_event = evaluate_note_userdata(&engine, r#"note({0x32, 48})"#)?;
    assert_eq!(
        note_event.notes,
        vec![new_note((None, "d4")), new_note((None, "c4"))]
    );

    // Note table
    assert!(evaluate_note_userdata(&engine, r#"note({volume = 0.5})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note({key = "xxx", volume = 0.5})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note({key = "C#1", volume = "abc"})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note({key = "C#1", volume = -1})"#).is_err());
    let note_event = evaluate_note_userdata(&engine, r#"note({key = "c8"})"#)?;
    assert_eq!(note_event.notes, vec![new_note((None, "c8"))]);
    let note_event = evaluate_note_userdata(&engine, r#"note({key = "G8", volume = 2})"#)?;
    assert_eq!(note_event.notes, vec![new_note((None, "g8", 2.0))]);

    // Note table or array
    let poly_note_event = evaluate_note_userdata(
        &engine,
        r#"note({{key = "C#1", volume = 0.5}, {key = "G2", volume = 0.75}, {}})"#,
    )?;
    assert_eq!(
        poly_note_event.notes,
        vec![
            new_note((None, "c#1", 0.5)),
            new_note((None, "g2", 0.75)),
            None
        ]
    );
    Ok(())
}

#[test]
fn note_chord() -> Result<(), Box<dyn std::error::Error>> {
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

    // Note chord
    assert!(evaluate_note_userdata(&engine, r#"note("c12'maj")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("j'maj")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4'invalid")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4'maj'")"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4'maj xx")"#).is_err());

    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c'maj")"#)?.notes,
        vec![
            new_note((None, "c4")),
            new_note((None, "e4")),
            new_note((None, "g4")),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c7'maj 0.2")"#)?.notes,
        vec![
            new_note((None, "c7", 0.2)),
            new_note((None, "e7", 0.2)),
            new_note((None, "g7", 0.2)),
        ]
    );
    Ok(())
}

#[test]
fn note_methods() -> Result<(), Box<dyn std::error::Error>> {
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

    // transpose
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "d4", "e4"):transpose(12)"#)?.notes,
        vec![
            new_note((None, "c5")),
            new_note((None, "d5")),
            new_note((None, "e5")),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "d4", "e4"):transpose({2, 4})"#)?.notes,
        vec![
            new_note((None, "d_4")),
            new_note((None, "f#4")),
            new_note((None, "e_4")),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "c4"):transpose({-1000, 1000})"#)?.notes,
        vec![new_note((None, 0x0_u8)), new_note((None, 0x7f_u8)),]
    );

    // with_volume
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume(1.0)"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume()"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume(-1)"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume({})"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume({"wurst"})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_volume({-1})"#).is_err());
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4 0.5", "d4 0.5", "e4 0.5"):with_volume(2.0)"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 2.0)),
            new_note((None, "d4", 2.0)),
            new_note((None, "e4", 2.0)),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4 0.5", "d4 0.5", "e4 0.5"):with_volume({2.0, 4.0})"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 2.0)),
            new_note((None, "d4", 4.0)),
            new_note((None, "e4", 0.5)),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4 0.5", "d4 0.5", "e4 0.5"):with_volume({2.0, 2.0, 2.0, 2.0})"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 2.0)),
            new_note((None, "d4", 2.0)),
            new_note((None, "e4", 2.0)),
        ]
    );

    // amplify
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4 0.5", "d4 0.5", "e4 0.5"):amplify(2.0)"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 1.0)),
            new_note((None, "d4", 1.0)),
            new_note((None, "e4", 1.0)),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4 0.5", "d4 0.5", "e4 0.5"):amplify({2.0, 4.0})"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 1.0)),
            new_note((None, "d4", 2.0)),
            new_note((None, "e4", 0.5)),
        ]
    );

    // with_panning
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning(1.0)"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning()"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning(-2)"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning({})"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning({"wurst"})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_panning({2})"#).is_err());
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "d4", "e4"):with_panning(-1.0)"#)?.notes,
        vec![
            new_note((None, "c4", 1.0, -1.0)),
            new_note((None, "d4", 1.0, -1.0)),
            new_note((None, "e4", 1.0, -1.0)),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(
            &engine,
            r#"note("c4", "d4", "e4"):with_panning({-1.0, 1.0})"#
        )?
        .notes,
        vec![
            new_note((None, "c4", 1.0, -1.0)),
            new_note((None, "d4", 1.0, 1.0)),
            new_note((None, "e4", 1.0, 0.0)),
        ]
    );

    // with_delay
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay(1.0)"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay()"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay(-1)"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay({})"#).is_ok());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay({"wurst"})"#).is_err());
    assert!(evaluate_note_userdata(&engine, r#"note("c4"):with_delay({2})"#).is_err());
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "d4", "e4"):with_delay(0.75)"#)?.notes,
        vec![
            new_note((None, "c4", 1.0, 0.0, 0.75)),
            new_note((None, "d4", 1.0, 0.0, 0.75)),
            new_note((None, "e4", 1.0, 0.0, 0.75)),
        ]
    );
    assert_eq!(
        evaluate_note_userdata(&engine, r#"note("c4", "d4", "e4"):with_delay({0.25, 0.5})"#)?.notes,
        vec![
            new_note((None, "c4", 1.0, 0.0, 0.25)),
            new_note((None, "d4", 1.0, 0.0, 0.5)),
            new_note((None, "e4", 1.0, 0.0)),
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

    // Note Sequence
    let note_sequence_event =
        evaluate_sequence_userdata(&engine, r#"sequence({"C#1 0.5"}, "---", "G_2")"#)?;
    assert_eq!(
        note_sequence_event.notes,
        vec![
            vec![new_note((instrument, "c#1", 0.5))],
            vec![None],
            vec![new_note((instrument, "g2", 1.0))]
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
                new_note((instrument, "c#1", 1.0)),
                None,
                new_note((instrument, "g2", 0.75)),
            ],
            vec![
                new_note((instrument, "a#5", 0.2)),
                None,
                new_note((instrument, "b1", 0.1))
            ]
        ]
    );

    let chord_sequence_event = evaluate_sequence_userdata(
        &engine, //
        r#"sequence("c'maj")"#,
    )?;
    assert_eq!(
        chord_sequence_event.notes,
        vec![vec![
            new_note((instrument, "c4")),
            new_note((instrument, "e4")),
            new_note((instrument, "g4")),
        ],]
    );

    let poly_chord_sequence_event = evaluate_sequence_userdata(
        &engine,
        r#"sequence( "c'maj", {"as5 0.2", "---", {key = "B_1", volume = 0.1}})"#,
    )?;
    assert_eq!(
        poly_chord_sequence_event.notes,
        vec![
            vec![
                new_note((instrument, "c4")),
                new_note((instrument, "e4")),
                new_note((instrument, "g4")),
            ],
            vec![
                new_note((instrument, "a#5", 0.2)),
                None,
                new_note((instrument, "b1", 0.1))
            ]
        ]
    );

    // with_xxx
    assert!(evaluate_sequence_userdata(
        &engine, //
        r#"sequence("c", "d", "f"):transpose(1)"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine, //
        r#"sequence("c'maj"):with_volume(2.0)"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine, //
        r#"sequence(12, 24, 48):with_panning(0.0)"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine,
        r#"sequence({key = "c"}, "d", "f"):with_delay(0.0)"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine, //
        r#"sequence("c", "d", "f"):transpose({1, 2})"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine,
        r#"sequence("c", "d", "f"):with_volume({2.0, 1.0})"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine,
        r#"sequence("c", "d", "f"):with_panning({0.0, 1.0})"#
    )
    .is_ok());
    assert!(evaluate_sequence_userdata(
        &engine,
        r#"sequence("c", "d", "f"):with_delay({0.0, 0.25})"#
    )
    .is_ok());

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
                    pattern = {1,0,1,0},
                    emit = "c6"
                }
            "#,
        )
        .eval::<LuaValue>()
        .unwrap();
    let mut beat_time_rhythm = beat_time_rhythm
        .as_userdata()
        .unwrap()
        .borrow::<BeatTimeRhythm>();
    assert!(beat_time_rhythm.is_ok());
    let mut beat_time_rhythm = beat_time_rhythm.as_mut().unwrap().clone();
    assert_eq!(beat_time_rhythm.step(), BeatTimeStep::Beats(0.5));
    assert_eq!(beat_time_rhythm.offset(), BeatTimeStep::Beats(2.0));
    assert_eq!(beat_time_rhythm.pattern(), vec![true, false, true, false]);
    let event = beat_time_rhythm.next();
    assert_eq!(
        event,
        Some((
            44100,
            Some(Event::NoteEvents(vec![Some(NoteEvent {
                instrument: None,
                note: Note::C6,
                volume: 1.0,
                panning: 0.0,
                delay: 0.0
            })]))
        ))
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
