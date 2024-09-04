---@meta
error("Do not try to execute this file. It's just a type definition file.")
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq InputParameter class.
---

----------------------------------------------------------------------------------------------------

---Unique id of the parameter. The id will be used in the `input` context table as key.
---@alias InputParameterId string

---Default boolean value.
---@alias InputParameterBooleanDefault boolean
---Default integer value. Must be in the specified value range.
---@alias InputParameterIntegerDefault integer
---Default number value. Must be in the specified value range.
---@alias InputParameterNumberDefault number
---Default string value. Must be a valid string within the specified value set.
---@alias InputParameterEnumDefault string

---Optional value range. When undefined (0.0 - 1.0)
---@alias InputParameterIntegerRange { [1]: integer, [2]: integer }
---Optional value range. When undefined (0 - 100)
---@alias InputParameterNumberRange { [1]: number, [2]: number }

---Optional name of the parameter as displayed to the user. When undefined, the id is used.
---@alias InputParameterName string

---Optional long description of the parameter describing what the parameter does.
---@alias InputParameterDescription string


----------------------------------------------------------------------------------------------------

---Opaque input parameter user data. Construct new input parameters via the `XXX_input(...)`
---functions. Input parameter values can then be accessed via function contexts in pattern,
---gate and emitter functions or generators.
---@see TriggerContext.inputs
---@class InputParameter : userdata
local InputParameter = {}

----------------------------------------------------------------------------------------------------

---Functions to create InputParamters.
parameter = {
    ---Creates an InputParameter with "boolean" Lua type with the given default value
    ---and other optional properties.
    ---@param id InputParameterId
    ---@param default InputParameterBooleanDefault
    ---@param name InputParameterName?
    ---@param description InputParameterDescription?
    ---@return InputParameter
    boolean = function(id, default, name, description) end,

    ---Creates an InputParameter with "integer" Lua type with the given default value
    ---and other optional properties.
    ---@param id InputParameterId
    ---@param default InputParameterIntegerDefault
    ---@param range InputParameterIntegerRange?
    ---@param name InputParameterName?
    ---@param description InputParameterDescription?
    ---@return InputParameter
    integer = function(id, default, range, name, description) end,

    ---Creates an InputParameter with "number" Lua type with the given default value
    ---and other optional properties.
    ---@param id InputParameterId
    ---@param default InputParameterNumberDefault
    ---@param range InputParameterNumberRange?
    ---@param name InputParameterName?
    ---@param description InputParameterDescription?
    ---@return InputParameter
    number = function(id, default, range, name, description) end,

    ---Creates an InputParameter with a "string" Lua type with the given default value,
    ---set of valid values to choose from and other optional properties.
    ---@param id InputParameterId
    ---@param default InputParameterEnumDefault
    ---@param values string[]
    ---@param name InputParameterName?
    ---@param description InputParameterDescription?
    ---@return InputParameter
    enum = function(id, default, values, name, description) end,
}
