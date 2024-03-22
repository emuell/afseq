use mlua::prelude::*;

use crate::prelude::*;

use super::{
    bad_argument_error,
    unwrap::{note_degree_from_value, note_event_from_value},
};

// ---------------------------------------------------------------------------------------------

impl LuaUserData for Scale {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> LuaResult<LuaTable> {
            lua.create_sequence_from(this.notes().iter().map(|n| LuaInteger::from(*n as u8)))
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "chord",
            |lua, this, args: LuaMultiValue| -> LuaResult<LuaTable> {
                let args = args.into_vec();
                // parse degree
                let mut degree = 1;
                if !args.is_empty() {
                    degree = note_degree_from_value(args.get(0).unwrap(), 1)?;
                }
                // parse count
                let mut count = 3;
                if args.len() >= 2 {
                    let count_error = || {
                        Err(bad_argument_error(
                            "Scale:chord",
                            "number_of_notes",
                            1,
                            "number of notes must be an integer in range [1, 5]",
                        ))
                    };
                    if let Some(value) = args.get(1).unwrap().as_usize() {
                        count = value;
                        if !(1..=5).contains(&count) {
                            return count_error();
                        }
                    } else {
                        return count_error();
                    }
                }
                let notes = this
                    .chord_from_degree(degree, count)
                    .iter()
                    .map(|n| LuaInteger::from(*n as u8))
                    .collect::<Vec<_>>();
                lua.create_sequence_from(notes)
            },
        );

        methods.add_method(
            "degree",
            |_lua, this, args: LuaMultiValue| -> LuaResult<LuaMultiValue> {
                let args = args.into_vec();
                let mut ret = Vec::new();
                for (arg_index, arg) in args.iter().enumerate() {
                    let degree = note_degree_from_value(arg, arg_index)?;
                    if let Some(note) = this.notes_iter().nth(degree - 1) {
                        ret.push(LuaValue::Integer(u8::from(note) as LuaInteger));
                    }
                }
                Ok(LuaMultiValue::from_vec(ret))
            },
        );

        methods.add_method(
            "fit",
            |_lua, this, args: LuaMultiValue| -> LuaResult<LuaMultiValue> {
                let args = args.into_vec();
                let mut ret = Vec::new();
                for (arg_index, arg) in args.iter().enumerate() {
                    if let Some(note_event) = note_event_from_value(arg, Some(arg_index))? {
                        let fit_note = this.transpose(note_event.note, 0);
                        ret.push(LuaValue::Integer(u8::from(fit_note) as LuaInteger));
                    } else {
                        ret.push(LuaValue::Nil);
                    }
                }
                Ok(LuaMultiValue::from_vec(ret))
            },
        );
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::bindings::*;

    fn new_test_engine() -> LuaResult<Lua> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            &BeatTimeBase {
                beats_per_min: 120.0,
                beats_per_bar: 4,
                samples_per_sec: 44100,
            },
        )?;
        timeout_hook.reset();
        Ok(lua)
    }
    #[test]
    fn scale() -> LuaResult<()> {
        let lua = new_test_engine()?;

        // Scale (note, mode_name)
        assert!(lua
            .load(r#"scale("c", "wurst")"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"scale("c", "harmonic minor")"#)
            .eval::<LuaValue>()
            .is_ok());
        assert_eq!(
            lua.load(r#"scale("c5", "natural major").notes"#)
                .eval::<Vec<LuaValue>>()
                .unwrap()
                .iter()
                .map(|v| v.as_i32().unwrap())
                .collect::<Vec<i32>>(),
            vec![60, 62, 64, 65, 67, 69, 71]
        );

        // Scale (note, interval)
        assert!(lua
            .load(r#"scale("c", {"wurst"})"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"scale("c", {0,1,2,4,5,6,7,8,9,10,11})"#)
            .eval::<LuaValue>()
            .is_ok());
        assert_eq!(
            lua.load(r#"scale("c5", {0,3,5,7,10}).notes"#)
                .eval::<Vec<LuaValue>>()
                .unwrap()
                .iter()
                .map(|v| v.as_i32().unwrap())
                .collect::<Vec<i32>>(),
            vec![60, 63, 65, 67, 70]
        );
        Ok(())
    }

    #[test]
    fn scale_chord() -> LuaResult<()> {
        let lua = new_test_engine()?;

        assert!(lua
            .load(
                r#" local cmin = scale("c4", "minor")
                return cmin:chord("x", 3)"#
            )
            .eval::<Vec<LuaValue>>()
            .is_err());
        assert!(lua
            .load(
                r#"local cmin = scale("c4", "minor")
                return cmin:chord("x", 8)"#
            )
            .eval::<Vec<LuaValue>>()
            .is_err());

        assert_eq!(
            lua.load(
                r#"local cmin = scale("c4", "minor")
                return cmin:chord("i", 5)"#
            )
            .eval::<Vec<LuaValue>>()?
            .iter()
            .map(|v| v.as_i32().unwrap())
            .collect::<Vec<i32>>(),
            vec![48, 51, 55, 58, 62]
        );

        Ok(())
    }

    #[test]
    fn scale_degree() -> LuaResult<()> {
        let lua = new_test_engine()?;

        assert!(lua
            .load(
                r#" local cmin = scale("c4", "minor")
                return cmin:degree("x")"#
            )
            .eval::<LuaMultiValue>()
            .is_err());
        assert!(lua
            .load(
                r#"local cmin = scale("c4", "minor")
                return cmin:degree(8)"#
            )
            .eval::<LuaMultiValue>()
            .is_err());

        assert_eq!(
            lua.load(
                r#"local cmin = scale("c4", "minor")
                return cmin:degree("i", "iii", "v")"#
            )
            .eval::<LuaMultiValue>()?
            .iter()
            .map(|v| v.as_i32().unwrap())
            .collect::<Vec<i32>>(),
            vec![48, 51, 55]
        );

        Ok(())
    }

    #[test]
    fn scale_fit() -> LuaResult<()> {
        let lua = new_test_engine()?;

        assert_eq!(
            lua.load(
                r#"local cmin = scale("c4", "minor")
                return cmin:fit("c4", 50, { key = 54 })"#
            )
            .eval::<LuaMultiValue>()?
            .into_vec()
            .iter()
            .map(|v| v.as_i32().unwrap())
            .collect::<Vec<i32>>(),
            vec![48, 50, 53]
        );

        Ok(())
    }
}
