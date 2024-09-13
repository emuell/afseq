use std::path::Path;

use lazy_static::lazy_static;
use mlua::prelude::*;

use crate::{bindings::compile_chunk, bindings::transpiler::Transpiler};

// -------------------------------------------------------------------------------------------------

pub(crate) struct FennelTranspiler {}

impl Transpiler for FennelTranspiler {
    fn transpile(file_path: &Path) -> LuaResult<String> {
        lazy_static! {
            static ref FENNEL_BYTECODE: LuaResult<Vec<u8>> =
                compile_chunk(include_str!("./fennel.lua"));
        }
        let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
        let fennel = match FENNEL_BYTECODE.as_ref() {
            Ok(bytecode) => lua
                .load(bytecode)
                .set_name("[inbuilt:fennel.lua]")
                .set_mode(mlua::ChunkMode::Binary)
                .call::<(), LuaTable>(()),
            Err(err) => Err(err.clone()),
        }?;
        let compile_function = fennel.get::<_, LuaFunction>("compile")?;
        // debug.traceback = fennel.traceback
        let file_handle = lua
            .load(format!(
                "return io.open([[{}]])",
                file_path.to_string_lossy()
            ))
            .eval::<LuaValue>()?;
        let lua_code = compile_function.call::<_, LuaString>((file_handle, LuaValue::Nil))?;
        Ok(lua_code.to_string_lossy().to_string())
    }
}
