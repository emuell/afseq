use ::mlua::Lua;
use ::rhai::Engine as RhaiEngine;

pub(crate) mod rhai_unwrap;
pub mod rhai;

// ---------------------------------------------------------------------------------------------

/// Create a new rhai engine with preloaded packages and our default configuation
pub fn new_rhai_engine() -> RhaiEngine {
    rhai::new_engine()
}
