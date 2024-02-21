use mlua::prelude::*;

use super::unwrap::*;
use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

// Sequence
#[derive(Clone, Debug)]
pub struct SequenceUserData {
    pub notes: Vec<Vec<Option<NoteEvent>>>,
}

impl SequenceUserData {
    pub fn from(args: LuaMultiValue, default_instrument: Option<InstrumentId>) -> LuaResult<Self> {
        // a single value, probably a sequence array
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
                    // add each sequence item as separate sequence event
                    notes.push(note_events_from_value(
                        arg,
                        Some(index),
                        default_instrument,
                    )?);
                }
                Ok(SequenceUserData { notes })
            } else {
                Ok(SequenceUserData {
                    notes: vec![note_events_from_value(arg, None, default_instrument)?],
                })
            }
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.push(note_events_from_value(
                    arg,
                    Some(index),
                    default_instrument,
                )?);
            }
            Ok(SequenceUserData { notes })
        }
    }
}

impl LuaUserData for SequenceUserData {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> LuaResult<LuaTable> {
            let sequence = lua.create_table()?;
            for (index, note_events) in this.notes.iter().enumerate() {
                let notes = lua.create_table()?;
                for (index, note_event) in note_events.iter().enumerate() {
                    if let Some(note_event) = note_event {
                        notes.set(index + 1, note_event.clone().into_lua(lua)?)?;
                    } else {
                        notes.set(index + 1, LuaValue::Table(lua.create_table()?))?;
                    }
                }
                sequence.set(index + 1, notes)?;
            }
            Ok(sequence)
        })
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("transpose", |lua, this, volume: LuaValue| {
            let steps = transpose_steps_array_from_value(lua, volume, this.notes.len())?;
            for (notes, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                for note in notes.iter_mut().flatten() {
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
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });
        methods.add_method_mut("amplify", |lua, this, value: LuaValue| {
            let volumes = volume_array_from_value(lua, value, this.notes.len())?;
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_panning", |lua, this, value: LuaValue| {
            let pannings = panning_array_from_value(lua, value, this.notes.len())?;
            for (notes, panning) in this.notes.iter_mut().zip(pannings) {
                for note in notes.iter_mut().flatten() {
                    note.panning = panning;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("with_delay", |lua, this, value: LuaValue| {
            let delays = delay_array_from_value(lua, value, this.notes.len())?;
            for (notes, delay) in this.notes.iter_mut().zip(delays) {
                for note in notes.iter_mut().flatten() {
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
    use crate::bindings::*;

    fn evaluate_sequence_userdata(lua: &Lua, expression: &str) -> LuaResult<SequenceUserData> {
        Ok(lua
            .load(expression)
            .eval::<LuaValue>()?
            .as_userdata()
            .ok_or(LuaError::RuntimeError("No user data".to_string()))?
            .borrow::<SequenceUserData>()?
            .clone())
    }

    #[test]
    fn sequence() -> Result<(), Box<dyn std::error::Error>> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        let instrument = Some(InstrumentId::from(76));
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
            instrument,
        )?;

        // reset timeout
        timeout_hook.reset();

        // Note Sequence
        let note_sequence_event =
            evaluate_sequence_userdata(&lua, r#"sequence({"C#1 0.5"}, "---", "G_2")"#)?;
        assert_eq!(
            note_sequence_event.notes,
            vec![
                vec![new_note((instrument, "c#1", 0.5))],
                vec![None],
                vec![new_note((instrument, "g2", 1.0))]
            ]
        );
        let poly_note_sequence_event = evaluate_sequence_userdata(
            &lua,
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
            &lua, //
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
            &lua,
            r#"sequence("c'maj", {"as5 0.2", "---", {key = "B_1", volume = 0.1}})"#,
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

        let note_sequence_event =
            evaluate_sequence_userdata(&lua, r#"sequence{note{"c"}, note("d", "e"), {"f"}}"#)?;
        assert_eq!(
            note_sequence_event.notes,
            vec![
                vec![new_note((instrument, "c")),],
                vec![new_note((instrument, "d")), new_note((instrument, "e")),],
                vec![new_note((instrument, "f")),]
            ]
        );

        Ok(())
    }

    #[test]
    fn sequence_methods() -> Result<(), Box<dyn std::error::Error>> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        let instrument = Some(InstrumentId::from(76));
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
            instrument,
        )?;

        // reset timeout
        timeout_hook.reset();

        // notes
        assert_eq!(
            evaluate_sequence_userdata(
                &lua,
                r#"sequence(sequence{{"c4 0.2 0.3 0.4", "d4"}, {}, {"e4"}}.notes)"#
            )?
            .notes,
            vec![
                vec![
                    new_note((instrument, "c4", 0.2, 0.3, 0.4)),
                    new_note((instrument, "d4")),
                ],
                vec![None],
                vec![new_note((instrument, "e4"))],
            ]
        );

        // with_xxx
        assert!(evaluate_sequence_userdata(
            &lua, //
            r#"sequence("c", "d", "f"):transpose(1)"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua, //
            r#"sequence("c'maj"):with_volume(2.0)"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua, //
            r#"sequence(12, 24, 48):with_panning(0.0)"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua,
            r#"sequence({key = "c"}, "d", "f"):with_delay(0.0)"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua, //
            r#"sequence("c", "d", "f"):transpose({1, 2})"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua,
            r#"sequence("c", "d", "f"):with_volume({2.0, 1.0})"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua,
            r#"sequence("c", "d", "f"):with_panning({0.0, 1.0})"#
        )
        .is_ok());
        assert!(evaluate_sequence_userdata(
            &lua,
            r#"sequence("c", "d", "f"):with_delay({0.0, 0.25})"#
        )
        .is_ok());

        Ok(())
    }
}
