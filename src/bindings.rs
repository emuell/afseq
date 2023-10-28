//! Script binding modules for the entire crate (currently Lua and Rhai)

use ::mlua::Lua;
use ::rhai::Engine as RhaiEngine;

pub mod rhai;
pub mod lua;

// ---------------------------------------------------------------------------------------------

/// Create a new raw rhai engine, configured for use in our bindings.
/// To actually use it for scripting, call [register_bindings](rhai::register_bindings) 
/// on the new engine.
pub fn new_rhai_engine() -> RhaiEngine {
    rhai::new_engine()
}

/// Create a new raw lua engine, configured for use in our bindings.
/// To actually use it for scripting, call [register_bindings](lua::register_bindings) 
/// on the new engine.
pub fn new_lua_engine() -> Lua {
    lua::new_engine()
}