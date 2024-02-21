use mlua::prelude::*;

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
}

// --------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;
    use crate::bindings::*;

    #[test]
    fn scale() {
        // create a new engine and register bindings
        let (mut lua, mut timeout_hook) = new_engine();
        register_bindings(
            &mut lua,
            &timeout_hook,
            BeatTimeBase {
                beats_per_min: 160.0,
                beats_per_bar: 6,
                samples_per_sec: 96000,
            },
            Some(InstrumentId::from(76)),
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
    }
}
