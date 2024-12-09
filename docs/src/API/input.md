
<!-- toc -->

# InputParameter {#InputParameter}  
> Opaque input parameter user data. Construct new input parameters via the `parameter.XXX(...)`
> functions.  



# Parameter {#Parameter}  
> Contains functions to construct new input parameters. Input parameter values can be accessed
> via functionn `contexts` in pattern, gate and emitter functions or generators.  

---  
## Functions
### boolean(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterBooleanDefault`](#InputParameterBooleanDefault), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#boolean}
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "boolean" Lua type with the given default value
> and other optional properties.
### integer(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterIntegerDefault`](#InputParameterIntegerDefault), range : [`InputParameterIntegerRange`](#InputParameterIntegerRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#integer}
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "integer" Lua type with the given default value
> and other optional properties.
### number(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterNumberDefault`](#InputParameterNumberDefault), range : [`InputParameterNumberRange`](#InputParameterNumberRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#number}
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "number" Lua type with the given default value
> and other optional properties.
### enum(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterEnumDefault`](#InputParameterEnumDefault), values : [`string`](../API/builtins/string.md)[], name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#enum}
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with a "string" Lua type with the given default value,
> set of valid values to choose from and other optional properties.  



---  
## Aliases  
### InputParameterBooleanDefault {#InputParameterBooleanDefault}
[`boolean`](../API/builtins/boolean.md)  
> Default boolean value.  
  
### InputParameterDescription {#InputParameterDescription}
[`string`](../API/builtins/string.md)  
> Optional long description of the parameter describing what the parameter does.  
  
### InputParameterEnumDefault {#InputParameterEnumDefault}
[`string`](../API/builtins/string.md)  
> Default string value. Must be a valid string within the specified value set.  
  
### InputParameterId {#InputParameterId}
[`string`](../API/builtins/string.md)  
> Unique id of the parameter. The id will be used in the `input` context table as key.  
  
### InputParameterIntegerDefault {#InputParameterIntegerDefault}
[`integer`](../API/builtins/integer.md)  
> Default integer value. Must be in the specified value range.  
  
### InputParameterIntegerRange {#InputParameterIntegerRange}
{ 1 : [`integer`](../API/builtins/integer.md), 2 : [`integer`](../API/builtins/integer.md) }  
> Optional value range. When undefined (0.0 - 1.0)  
  
### InputParameterName {#InputParameterName}
[`string`](../API/builtins/string.md)  
> Optional name of the parameter as displayed to the user. When undefined, the id is used.  
  
### InputParameterNumberDefault {#InputParameterNumberDefault}
[`number`](../API/builtins/number.md)  
> Default number value. Must be in the specified value range.  
  
### InputParameterNumberRange {#InputParameterNumberRange}
{ 1 : [`number`](../API/builtins/number.md), 2 : [`number`](../API/builtins/number.md) }  
> Optional value range. When undefined (0 - 100)  
  



