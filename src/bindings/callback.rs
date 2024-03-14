use mlua::prelude::*;

// -------------------------------------------------------------------------------------------------

/// Lazily evaluates a lua function, the first time it's called, to either use it as a generator,
/// a function which returns a function, or directly as it is.
///
/// When calling the function the signature of the function is `fn(context): LuaResult`;
///
/// The passed context is created as an empty table with the function, and should be filled up
/// with values in the client who uses the function.
///
/// By memorizing the original generator function and environment, it also can be reset to its
/// initial state by calling the original generator function again to fetch a new freshly
/// initialized function.
///
/// TODO: Upvalues of generators or simple functions could actuially be collected and restored
/// too, but this uses debug functionality and may break some upvalues.
#[derive(Debug, Clone)]
pub(crate) struct LuaFunctionCallback {
    environment: Option<LuaOwnedTable>,
    context: LuaOwnedTable,
    generator: Option<LuaOwnedFunction>,
    function: LuaOwnedFunction,
    initialized: bool,
}

impl LuaFunctionCallback {
    pub fn new(lua: &Lua, function: LuaFunction<'_>) -> LuaResult<Self> {
        // create an empty context and memorize the function without calling it
        let context = lua.create_table()?.into_owned();
        let environment = function.environment().map(|env| env.into_owned());
        let generator = None;
        let function = function.into_owned();
        let initialized = false;
        Ok(Self {
            environment,
            context,
            generator,
            function,
            initialized,
        })
    }

    /// Mut access to the function context that is passed as one and only argument to the
    ///  wrapped function, when evaluating it.
    pub fn context(&mut self) -> &mut LuaOwnedTable {
        &mut self.context
    }

    /// Name of the inner function. Usually will be an annonymous function.
    pub fn name(&self) -> String {
        self.function
            .to_ref()
            .info()
            .name
            .unwrap_or("annonymous function".to_string())
    }

    /// Call the function with our context as argument and return the result as LuaValue.
    /// Fetches inner functions from generators, if this is the first call.
    pub fn call(&mut self) -> LuaResult<LuaValue> {
        if !self.initialized {
            self.initialized = true;
            let function = self.function.clone();
            let result = function.call::<_, LuaValue>(self.context.to_ref())?;
            if let Some(inner_function) = result.as_function() {
                // function returned a function -> is an iterator. use inner function instead.
                let function_environment = self
                    .function
                    .to_ref()
                    .environment()
                    .map(|env| env.into_owned());
                let function_generator = Some(self.function.clone());
                self.environment = function_environment;
                self.generator = function_generator;
                self.function = inner_function.clone().into_owned();
            } else {
                // function returned not a function. use this function directly.
                self.environment = None;
                self.generator = None;
            }
        }
        self.function.call(self.context.to_ref())
    }

    /// Reset the function's environment and get a new fresh function from a generator,
    /// if the original function is a generator function.
    pub fn reset(&mut self) -> LuaResult<()> {
        // resetting only is necessary when we got initialized
        if self.initialized {
            if let Some(function_generator) = &self.generator {
                // restore generator environment
                if let Some(env) = &self.environment {
                    function_generator.to_ref().set_environment(env.to_ref())?;
                }
                // then fetch a new fresh function from the generator
                let value = function_generator
                    .to_ref()
                    .call::<_, LuaValue>(self.context.to_ref())?;
                if let Some(function) = value.as_function() {
                    self.function = function.clone().into_owned();
                } else {
                    return Err(LuaError::runtime(format!(
                        "Failed to reset custom generator function '{}' \
                         Expected a function as return value, got a '{}'",
                        self.name(),
                        value.type_name()
                    )));
                }
            }
        }
        Ok(())
    }
}
