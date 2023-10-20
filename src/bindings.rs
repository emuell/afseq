use ::mlua::Lua;
use ::rhai::Engine as RhaiEngine;

pub mod rhai;
pub mod lua;

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_rhai_engine() -> RhaiEngine {
    rhai::new_engine()
}

/// Create a new lua engine with preloaded packages and our default configuation
pub fn new_lua_engine() -> Lua {
    lua::new_engine()
}