use std::{cell::RefCell, path::Path};

use mlua::prelude::*;

use super::Transpiler;

// -------------------------------------------------------------------------------------------------

pub(crate) struct ErdeTranspiler {}

impl Transpiler for ErdeTranspiler {
    fn transpile<'a, P: Into<Option<&'a Path>>>(
        file_contents: &str,
        _file_path: P,
    ) -> LuaResult<String> {
        // get cached compile function
        thread_local! {
            static FENNEL: RefCell<LuaResult<LuaOwnedFunction>> = {
                let try_create = || -> LuaResult<LuaOwnedFunction> {
                    let lua = unsafe { Lua::unsafe_new_with(LuaStdLib::ALL, LuaOptions::default()) };
                    let erde = lua.load(include_str!("./erde.lua"))
                        .set_name("[inbuilt:erde.lua]")
                        .call::<(), LuaTable>(())?;
                    let traceback_function = erde.get::<_, LuaFunction>("traceback")?;
                    lua.globals()
                        .get::<_, LuaTable>("debug")?
                        .set("traceback", traceback_function)?;
                    let compile_function = erde.get::<_, LuaFunction>("compile")?;
                    Ok(compile_function.into_owned())
                };
                RefCell::new(try_create().map_err(|err|
                    LuaError::runtime(format!("Failed to load lua transpiler: {}", err))))
            };
        }
        let compile_function = FENNEL.with_borrow(|fennel| fennel.clone())?;
        // compile file
        let compile_options = LuaValue::Nil;
        let lua_code = compile_function
            .to_ref()
            .call::<_, LuaString>((file_contents, compile_options))
            .map_err(|err| LuaError::SyntaxError {
                message: match err {
                    LuaError::RuntimeError(str) => str.clone(),
                    _ => err.to_string(),
                },
                incomplete_input: false,
            })?;
        // return compiled code
        Ok(lua_code.to_string_lossy().to_string())
    }
}
