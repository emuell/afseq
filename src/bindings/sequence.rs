use mlua::prelude::*;

use crate::prelude::*;
use super::unwrap::*;

// ---------------------------------------------------------------------------------------------

// Sequence
#[derive(Clone, Debug)]
pub struct SequenceUserData {
    pub notes: Vec<Vec<Option<NoteEvent>>>,
}

impl SequenceUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        // a single value, probably a sequence array
        let args = args.into_vec();
        if args.len() == 1 {
            let arg = args
                .first()
                .ok_or(mlua::Error::RuntimeError(
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
