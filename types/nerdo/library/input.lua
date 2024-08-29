---@meta
error("Do not try to execute this file. It's just a type definition file.")
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq InputParameter class.
---

----------------------------------------------------------------------------------------------------

---Unique id of the parameter. The id will be used in the `input` context table as key.
---@alias InputParameterId string

---Optional name of the parameter as displayed to the user. When undefined the id is used.
---@alias InputParameterName string

---Optional long description of the parameter describing what the parameter does.
---@alias InputParameterDescription string

---Valid value range. Default: (0 - 1)
---@alias InputParameterIntegerRange { [1]: integer, [2]: integer }
---Valid value range. Default: (0 - 1)
---@alias InputParameterFloatRange { [1]: number, [2]: number }

---Default value. Default: false
---@alias InputParameterBooleanDefault boolean
---Default value. Default: 0
---@alias InputParameterIntegerDefault integer
---Default value. Default: 0.0
---@alias InputParameterFloatDefault number

----------------------------------------------------------------------------------------------------

---Opaque input parameter user data. Construct new input parameters via the `XXX_input(...)`
---functions. Input parameter values can then be accessed via function contexts in pattern,
---gate and emitter functions or generators. 
---@see TriggerContext.inputs
---@class InputParameter : userdata
local InputParameter = {}

----------------------------------------------------------------------------------------------------

---Creates a InputParameter with "boolean" Lua type and the given other properties.
---@param id InputParameterId
---@param default InputParameterBooleanDefault
---@param name InputParameterName?
---@param description InputParameterDescription?
---@return InputParameter
function boolean_input(id, default, name, description) end

---Creates a InputParameter with  "integer" Lua type and the given other properties.
---@param id InputParameterId
---@param min_max InputParameterIntegerRange
---@param default InputParameterIntegerDefault
---@param name InputParameterName?
---@param description InputParameterDescription?
---@return InputParameter
function integer_input(id, min_max, default, name, description) end

---Creates a InputParameter with "number" Lua type and the given other properties.
---@param id InputParameterId
---@param min_max InputParameterFloatRange
---@param default InputParameterFloatDefault
---@param name InputParameterName?
---@param description InputParameterDescription?
---@return InputParameter
function number_input(id, min_max, default, name, description) end
