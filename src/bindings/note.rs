use mlua::prelude::*;

use super::unwrap::*;
use crate::{event::NoteEvent, note::Note};

// ---------------------------------------------------------------------------------------------

/// Note Userdata in bindings
#[derive(Clone, Debug)]
pub struct NoteUserData {
    pub notes: Vec<Option<NoteEvent>>,
}

impl NoteUserData {
    pub fn from(args: LuaMultiValue) -> LuaResult<Self> {
        // a single value, probably a sequence
        let args = args.into_vec();
        if args.len() == 1 {
            let arg = args
                .first()
                .ok_or(LuaError::RuntimeError(
                    "Failed to access table content".to_string(),
                ))
                .cloned()?;
            if let Some(sequence) = sequence_from_value(&arg.clone()) {
                let mut notes = vec![];
                for (index, arg) in sequence.into_iter().enumerate() {
                    // flatten sequence events into a single array
                    notes.append(&mut note_events_from_value(arg, Some(index))?);
                }
                Ok(NoteUserData { notes })
            } else {
                Ok(NoteUserData {
                    notes: note_events_from_value(arg, None)?,
                })
            }
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.append(&mut note_events_from_value(arg, Some(index))?);
            }
            Ok(NoteUserData { notes })
        }
    }
}

impl LuaUserData for NoteUserData {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> LuaResult<LuaTable> {
            let sequence = lua.create_table()?;
            for (index, note_event) in this.notes.iter().enumerate() {
                if let Some(note_event) = note_event {
                    sequence.set(index + 1, note_event.clone().into_lua(lua)?)?;
                } else {
                    sequence.set(index + 1, LuaValue::Table(lua.create_table()?))?;
                }
            }
            Ok(sequence)
        })
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("transpose", |lua, this, value: LuaValue| {
            let steps = transpose_steps_array_from_value(lua, value, this.notes.len())?;
            for (note, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                if let Some(note) = note {
                    if note.note.is_note_on() {
                        let transposed_note = (u8::from(note.note) as i32 + step).max(0).min(0x7f);
                        note.note = Note::from(transposed_note as u8);
                    }
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_volume", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("amplify", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("with_panning", |lua, this, value: LuaValue| {
            let pannings = panning_array_from_value(lua, value, this.notes.len())?;
            for (note, panning) in this.notes.iter_mut().zip(pannings.into_iter()) {
                if let Some(note) = note {
                    note.panning = panning;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_delay", |lua, this, value: LuaValue| {
            let delays = delay_array_from_value(lua, value, this.notes.len())?;
            for (note, delay) in this.notes.iter_mut().zip(delays.into_iter()) {
                if let Some(note) = note {
                    note.delay = delay;
                }
            }
            Ok(this.clone())
        });
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::{bindings::*, event::new_note};

    fn evaluate_note_userdata(lua: &Lua, expression: &str) -> LuaResult<NoteUserData> {
        Ok(lua
            .load(expression)
            .eval::<LuaValue>()?
            .as_userdata()
            .ok_or(LuaError::RuntimeError("No user data".to_string()))?
            .borrow::<NoteUserData>()?
            .clone())
    }

    #[test]
    fn note() -> LuaResult<()> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )?;

        // reset timeout
        timeout_hook.reset();

        // Empty Note
        let note_event = evaluate_note_userdata(&lua, r#"note("---")"#)?;
        assert_eq!(note_event.notes, vec![None]);

        // Note Off
        assert!(evaluate_note_userdata(&lua, r#"note("off")"#).is_ok());
        let note_event = evaluate_note_userdata(&lua, r#"note("OFF")"#)?;
        assert_eq!(note_event.notes, vec![new_note(Note::OFF)]);

        // Note string
        assert!(evaluate_note_userdata(&lua, r#"note("X#1")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("0.5")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 -0.5")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1", 0.5)"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1")"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 #a")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 #-10")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 v-2.0")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 p-1.0")"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 d-1.0")"#).is_err());
        let note_event = evaluate_note_userdata(&lua, r#"note("C#1 #2 v0.5 p0.1 d0.2")"#)?;
        assert_eq!(
            note_event.notes,
            vec![new_note((Note::Cs1, InstrumentId::from(2), 0.5, 0.1, 0.2))]
        );
        let note_event = evaluate_note_userdata(&lua, r#"note("C#1 d0.2")"#)?;
        assert_eq!(
            note_event.notes,
            vec![new_note((Note::Cs1, None, 1.0, 0.0, 0.2))]
        );

        // Note string array
        assert!(evaluate_note_userdata(&lua, r#"note({"X#1"})"#).is_err());
        let note_event = evaluate_note_userdata(&lua, r#"note({"C#1"})"#)?;
        assert_eq!(note_event.notes, vec![new_note("c#1")]);

        assert!(evaluate_note_userdata(&lua, r#"note("X#1")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("C#1 abc")"#).is_err());
        let note_event = evaluate_note_userdata(&lua, r#"note({"C#1 v0.5", "C5"})"#)?;
        assert_eq!(
            note_event.notes,
            vec![new_note(("c#1", None, 0.5)), new_note(("c5", None, 1.0))]
        );

        // Note int
        let note_event = evaluate_note_userdata(&lua, r#"note(0x32)"#)?;
        assert_eq!(note_event.notes, vec![new_note("d4")]);

        // Note int array
        let note_event = evaluate_note_userdata(&lua, r#"note({0x32, 48})"#)?;
        assert_eq!(note_event.notes, vec![new_note("d4"), new_note("c4")]);

        // Note table
        assert!(evaluate_note_userdata(&lua, r#"note({volume = 0.5})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note({key = "xxx", volume = 0.5})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note({key = "C#1", volume = "abc"})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note({key = "C#1", volume = -1})"#).is_err());
        let note_event = evaluate_note_userdata(&lua, r#"note({key = "c8"})"#)?;
        assert_eq!(note_event.notes, vec![new_note("c8")]);
        let note_event = evaluate_note_userdata(&lua, r#"note({key = "G8", volume = 2})"#)?;
        assert_eq!(note_event.notes, vec![new_note(("g8", None, 2.0))]);

        // Note table or array
        let poly_note_event = evaluate_note_userdata(
            &lua,
            r#"note({{key = "C#1", volume = 0.5}, {key = "G2", volume = 0.75}, {}})"#,
        )?;
        assert_eq!(
            poly_note_event.notes,
            vec![
                new_note(("c#1", None, 0.5)),
                new_note(("g2", None, 0.75)),
                None
            ]
        );
        Ok(())
    }

    #[test]
    fn note_chord() -> LuaResult<()> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )?;

        // reset timeout
        timeout_hook.reset();

        // Note chord
        assert!(evaluate_note_userdata(&lua, r#"note("c12'maj")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("j'maj")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4'invalid")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4'maj'")"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4'maj xx")"#).is_err());

        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c'maj")"#)?.notes,
            vec![new_note("c4"), new_note("e4"), new_note("g4"),]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c7'maj v0.2")"#)?.notes,
            vec![
                new_note(("c7", None, 0.2)),
                new_note(("e7", None, 0.2)),
                new_note(("g7", None, 0.2)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4'm v0.2", "c7")"#)?.notes,
            vec![
                new_note(("c4", None, 0.2)),
                new_note(("d#4", None, 0.2)),
                new_note(("g4", None, 0.2)),
                new_note("c7"),
            ]
        );

        Ok(())
    }

    #[test]
    fn note_methods() -> LuaResult<()> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )?;

        // reset timeout
        timeout_hook.reset();

        // notes
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note(note("c4 #100 v0.2 p0.3 d0.4", "", "e4").notes)"#
            )?
            .notes,
            vec![
                new_note(("c4", InstrumentId::from(100), 0.2, 0.3, 0.4)),
                None,
                new_note("e4"),
            ]
        );

        // transpose
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):transpose(12)"#)?.notes,
            vec![new_note("c5"), new_note("d5"), new_note("e5"),]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):transpose({2, 4})"#)?.notes,
            vec![new_note("d_4"), new_note("f#4"), new_note("e_4"),]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "c4"):transpose({-1000, 1000})"#)?.notes,
            vec![new_note(0x0_u8), new_note(0x7f_u8),]
        );

        // with_volume
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume(1.0)"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume()"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume(-1)"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume({})"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume({"wurst"})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_volume({-1})"#).is_err());
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note("c4 v0.5", "d4 v0.5", "e4 v0.5"):with_volume(2.0)"#
            )?
            .notes,
            vec![
                new_note(("c4", None, 2.0)),
                new_note(("d4", None, 2.0)),
                new_note(("e4", None, 2.0)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note("c4 v0.5", "d4 v0.5", "e4 v0.5"):with_volume({2.0, 4.0})"#
            )?
            .notes,
            vec![
                new_note(("c4", None, 2.0)),
                new_note(("d4", None, 4.0)),
                new_note(("e4", None, 0.5)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note("c4 v0.5", "d4 v0.5", "e4 v0.5"):with_volume({2.0, 2.0, 2.0, 2.0})"#
            )?
            .notes,
            vec![
                new_note(("c4", None, 2.0)),
                new_note(("d4", None, 2.0)),
                new_note(("e4", None, 2.0)),
            ]
        );

        // amplify
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note("c4 v0.5", "d4 v0.5", "e4 v0.5"):amplify(2.0)"#
            )?
            .notes,
            vec![
                new_note(("c4", None, 1.0)),
                new_note(("d4", None, 1.0)),
                new_note(("e4", None, 1.0)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(
                &lua,
                r#"note("c4 v0.5", "d4 v0.5", "e4 v0.5"):amplify({2.0, 4.0})"#
            )?
            .notes,
            vec![
                new_note(("c4", None, 1.0)),
                new_note(("d4", None, 2.0)),
                new_note(("e4", None, 0.5)),
            ]
        );

        // with_panning
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning(1.0)"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning()"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning(-2)"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning({})"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning({"wurst"})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_panning({2})"#).is_err());
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):with_panning(-1.0)"#)?.notes,
            vec![
                new_note(("c4", None, 1.0, -1.0)),
                new_note(("d4", None, 1.0, -1.0)),
                new_note(("e4", None, 1.0, -1.0)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):with_panning({-1.0, 1.0})"#)?
                .notes,
            vec![
                new_note(("c4", None, 1.0, -1.0)),
                new_note(("d4", None, 1.0, 1.0)),
                new_note(("e4", None, 1.0, 0.0)),
            ]
        );

        // with_delay
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay(1.0)"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay()"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay(-1)"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay({})"#).is_ok());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay({"wurst"})"#).is_err());
        assert!(evaluate_note_userdata(&lua, r#"note("c4"):with_delay({2})"#).is_err());
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):with_delay(0.75)"#)?.notes,
            vec![
                new_note(("c4", None, 1.0, 0.0, 0.75)),
                new_note(("d4", None, 1.0, 0.0, 0.75)),
                new_note(("e4", None, 1.0, 0.0, 0.75)),
            ]
        );
        assert_eq!(
            evaluate_note_userdata(&lua, r#"note("c4", "d4", "e4"):with_delay({0.25, 0.5})"#)?
                .notes,
            vec![
                new_note(("c4", None, 1.0, 0.0, 0.25)),
                new_note(("d4", None, 1.0, 0.0, 0.5)),
                new_note(("e4", None, 1.0, 0.0)),
            ]
        );

        Ok(())
    }
}
