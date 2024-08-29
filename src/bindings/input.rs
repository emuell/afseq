use mlua::prelude::*;

use crate::InputParameter;

// ---------------------------------------------------------------------------------------------

/// Opaque Lua Userdata impl for an InputParameter.
pub(crate) struct InputParameterUserData {
    pub(crate) parameter: InputParameter,
}

// Use default IntoLua impl for LuaUserData
impl LuaUserData for InputParameterUserData {}

// ---------------------------------------------------------------------------------------------

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
    fn inputs() -> LuaResult<()> {
        let lua = new_test_engine()?;

        // boolean_input
        assert!(lua
            .load(r#"boolean_input("name", "off")"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"boolean_input("name", false, {})"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"boolean_input("name", false, "Fancy Name", "Fancy Description")"#)
            .eval::<LuaValue>()
            .is_ok());

        // integer_input
        assert!(lua
            .load(r#"integer_input("name", false)"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"integer_input("name", {1, 20}, 20.5)"#) // not an integer
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"integer_input("name", {1, 20}, 50)"#) // out of range
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"integer_input("name", {-20, 20}, 0, "Fancy Name", "Fancy Description")"#)
            .eval::<LuaValue>()
            .is_ok());

        // number_input
        assert!(lua
            .load(r#"number_input("name", false)"#)
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"number_input("name", {1, 20}, 50)"#) // out of range
            .eval::<LuaValue>()
            .is_err());
        assert!(lua
            .load(r#"number_input("name", {-20, 20}, 0, "Fancy Name", "Fancy Description")"#)
            .eval::<LuaValue>()
            .is_ok());
        assert!(lua
            .load(r#"number_input("name", {-20.5, 20.5}, 0.5, "Fancy Name", "Fancy Description")"#)
            .eval::<LuaValue>()
            .is_ok());
        Ok(())
    }
}
