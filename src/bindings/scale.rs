use mlua::prelude::*;

use crate::prelude::*;

// ---------------------------------------------------------------------------------------------

impl LuaUserData for Scale {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("notes", |lua, this| -> mlua::Result<LuaTable> {
            lua.create_sequence_from(
                this.notes()
                    .iter()
                    .map(|n| LuaValue::Integer(*n as u8 as i64)),
            )
        })
    }
}

