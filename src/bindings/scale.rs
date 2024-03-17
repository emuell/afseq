use mlua::prelude::*;

use super::bad_argument_error;
use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

impl LuaUserData for Scale {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> LuaResult<LuaTable> {
            lua.create_sequence_from(
                this.notes()
                    .iter()
                    .map(|n| LuaValue::Integer(*n as u8 as LuaInteger)),
            )
        })
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "chord",
            |lua, this, args: LuaMultiValue| -> LuaResult<LuaTable> {
                // a single value, probably a sequence
                let args = args.into_vec();
                // parse degree
                let mut degree = 1;
                if !args.is_empty() {
                    let degree_error = || {
                        Err(bad_argument_error(
                            "Scale:chord",
                            "degree",
                            1,
                            "degree must be an integer or roman number string in range [1, 7] \
                                      (e.g. 3, 5, or 'iii' or 'V')",
                        ))
                    };
                    if let Some(value) = args.get(0).unwrap().as_integer() {
                        degree = value as usize;
                        if !(1..=7).contains(&degree) {
                            return degree_error();
                        }
                    } else if let Some(value) = args.get(0).unwrap().as_str() {
                        match value.to_lowercase().as_str() {
                            "i" => degree = 1,
                            "ii" => degree = 2,
                            "iii" => degree = 3,
                            "iv" => degree = 4,
                            "v" => degree = 5,
                            "vi" => degree = 6,
                            "vii" => degree = 7,
                            _ => return degree_error(),
                        }
                    } else {
                        return degree_error();
                    }
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
                    if let Some(value) = args.get(1).unwrap().as_integer() {
                        count = value as usize;
                        if !(1..=5).contains(&count) {
                            return count_error();
                        }
                    } else {
                        return count_error();
                    }
                }
                let notes = this
                    .chord_from_degree(degree, count)
                    .into_iter()
                    .map(|n| n as u8 as LuaInteger)
                    .collect::<Vec<_>>();
                lua.create_sequence_from(notes)
            },
        )
    }
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::bindings::*;

    #[test]
    fn scale() -> LuaResult<()> {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine()?;
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
        )
        .unwrap();

        // reset timeout
        timeout_hook.reset();

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
}
