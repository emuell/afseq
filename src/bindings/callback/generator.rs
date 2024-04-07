use std::{borrow::Cow, fmt::Debug};

use mlua::prelude::*;

use super::LuaCallback;
use crate::{BeatTimeBase, PulseIterItem};

// -------------------------------------------------------------------------------------------------

/// Wraps a lua fun generator into a callable function.
///
/// A fun iterator is a table with the following required keys:
/// ```{ gen: fn(param, state) -> any, param: any, state: any}```
/// See <https://luafun.github.io/> for more information.
///
/// Errors from callbacks should be handled by calling `self.handle_error` so external clients
/// can deal with them later, as apropriate.
///
/// Fun iterators don't make use of the LuaCallback's external state, so context setters are not
/// implemented.
///
/// NB: a generator's `param` and `state` can be any lua value, so we're memorizing it in a table
/// along with the generator's initial state in fields `"param"`, `"current"` and `"initial"`
/// as `LuaValue` can not be converted `into_owned`. This avoids memorizing the generator's lua
/// instance.
#[derive(Debug, Clone)]
pub(crate) struct LuaGeneratorCallback {
    generator: LuaOwnedFunction,
    state: LuaOwnedTable,
}

impl LuaGeneratorCallback {
    pub fn new(lua: &Lua, table: LuaTable) -> LuaResult<Self> {
        // validate generator table
        let generator = table.get::<_, LuaFunction>("gen")?.into_owned();
        let param = table.get::<_, LuaValue>("param")?;
        let initial_state = table.get::<_, LuaValue>("state")?;
        let current_state = {
            if let Some(initial_state) = initial_state.as_table() {
                // shallow clone initial state
                let current_state = lua.create_table()?;
                Self::clone_table(initial_state, &current_state)?;
                LuaValue::Table(current_state)
            } else {
                initial_state.clone()
            }
        };
        let state = lua
            .create_table_from([
                ("param", param),
                ("current", current_state),
                ("initial", initial_state),
            ])?
            .into_owned();
        Ok(Self { generator, state })
    }

    fn clone_table<'lua>(source: &'lua LuaTable, dest: &'lua LuaTable) -> LuaResult<()> {
        dest.clear()?;
        for pair in source.clone().pairs::<LuaValue, LuaValue>() {
            let (key, value) = pair?;
            dest.set(key, value)?;
        }
        Ok(())
    }
}

impl LuaCallback for LuaGeneratorCallback {
    fn set_context_time_base(&mut self, _time_base: &BeatTimeBase) -> LuaResult<()> {
        Ok(()) // unused
    }

    fn set_context_external_data(&mut self, _data: &[(Cow<str>, f64)]) -> LuaResult<()> {
        Ok(()) // unused
    }

    fn set_context_pulse_step(
        &mut self,
        _pulse_step: usize,
        _pulse_time_step: f64,
        _pulse_pattern_length: usize,
    ) -> LuaResult<()> {
        Ok(()) // unused
    }

    fn set_context_pulse_value(&mut self, _pulse: PulseIterItem) -> LuaResult<()> {
        Ok(()) // unused
    }

    fn set_context_step(&mut self, _step: usize) -> LuaResult<()> {
        Ok(()) // unused
    }

    fn name(&self) -> String {
        self.generator
            .to_ref()
            .info()
            .name
            .unwrap_or("annonymous function".to_string())
    }

    fn call(&mut self) -> LuaResult<Option<LuaValue>> {

        // Call the generator and return the result as `LuaValue`.
        // returns `None` when the generator is finished, else a value, which is probably nil.
        //
        // ```lua
        // -- with g.gen, g.state, g.param:
        // local state = g.state
        // while state do
        //   local new_state, value = g.gen(g.param, state)
        //   if new_state ~= nil then
        //     consume(value)
        //   end
        //   state = new_state
        // end
        // ```

        let param = self.state.to_ref().raw_get::<_, LuaValue>("param")?;
        let state = self.state.to_ref().raw_get::<_, LuaValue>("current")?;
        if state.is_nil() {
            Ok(None)
        } else {
            let mut result = self
                .generator
                .call::<_, LuaMultiValue>((param, state))?
                .into_vec();
            let value = result.pop().unwrap_or(LuaValue::Nil);
            let new_state = result.pop().unwrap_or(LuaValue::Nil);
            if new_state.is_nil() {
                self.state
                    .to_ref()
                    .raw_set::<_, LuaValue>("current", new_state)?;
                Ok(None)
            } else {
                self.state
                    .to_ref()
                    .raw_set::<_, LuaValue>("current", new_state)?;
                Ok(Some(value))
            }
        }
    }

    fn reset(&mut self) -> LuaResult<()> {
        let state = self.state.to_ref();
        let initial_state = state.get::<_, LuaValue>("initial")?;
        if let Some(initial_state) = initial_state.as_table() {
            // shallow clone initial state
            let current_state = state.get::<_, LuaValue>("current")?;
            let current_state = current_state.as_table().unwrap();
            Self::clone_table(initial_state, current_state)?;
        } else {
            state.raw_set("current", initial_state)?;
        }

        Ok(())
    }

    fn duplicate(&self) -> Box<dyn LuaCallback> {
        Box::new(self.clone())
    }
}
