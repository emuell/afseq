# Parameter  
> Create InputParameters.  

<!-- toc -->
  

---  
## Functions
### boolean(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterBooleanDefault`](#InputParameterBooleanDefault), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#boolean}
`->`[`InputParameter`](../API/InputParameter.md)  

> Creates an InputParameter with "boolean" Lua type with the given default value
> and other optional properties.
### enum(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterEnumDefault`](#InputParameterEnumDefault), values : [`string`](../API/builtins/string.md)[], name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#enum}
`->`[`InputParameter`](../API/InputParameter.md)  

> Creates an InputParameter with a "string" Lua type with the given default value,
> set of valid values to choose from and other optional properties.
### integer(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterIntegerDefault`](#InputParameterIntegerDefault), range : [`InputParameterIntegerRange`](#InputParameterIntegerRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#integer}
`->`[`InputParameter`](../API/InputParameter.md)  

> Creates an InputParameter with "integer" Lua type with the given default value
> and other optional properties.
### number(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterNumberDefault`](#InputParameterNumberDefault), range : [`InputParameterNumberRange`](#InputParameterNumberRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md)) {#number}
`->`[`InputParameter`](../API/InputParameter.md)  

> Creates an InputParameter with "number" Lua type with the given default value
> and other optional properties.  

