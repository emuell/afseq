# parameter
<!-- toc -->
# Parameter<a name="Parameter"></a>  
> Opaque parameter user data. Construct new parameters via the `parameter.XXX(...)`
> functions.  

---  
## Functions
### boolean(id : [`ParameterId`](#ParameterId), default : [`ParameterBooleanDefault`](#ParameterBooleanDefault), name : [`ParameterName`](#ParameterName)[`?`](../API/builtins/nil.md), description : [`ParameterDescription`](#ParameterDescription)[`?`](../API/builtins/nil.md))<a name="boolean"></a>
`->`[`Parameter`](../API/parameter.md#Parameter)  

> Creates an Parameter with "boolean" Lua type with the given default value
> and other optional properties.
### integer(id : [`ParameterId`](#ParameterId), default : [`ParameterIntegerDefault`](#ParameterIntegerDefault), range : [`ParameterIntegerRange`](#ParameterIntegerRange)[`?`](../API/builtins/nil.md), name : [`ParameterName`](#ParameterName)[`?`](../API/builtins/nil.md), description : [`ParameterDescription`](#ParameterDescription)[`?`](../API/builtins/nil.md))<a name="integer"></a>
`->`[`Parameter`](../API/parameter.md#Parameter)  

> Creates an Parameter with "integer" Lua type with the given default value
> and other optional properties.
### number(id : [`ParameterId`](#ParameterId), default : [`ParameterNumberDefault`](#ParameterNumberDefault), range : [`ParameterNumberRange`](#ParameterNumberRange)[`?`](../API/builtins/nil.md), name : [`ParameterName`](#ParameterName)[`?`](../API/builtins/nil.md), description : [`ParameterDescription`](#ParameterDescription)[`?`](../API/builtins/nil.md))<a name="number"></a>
`->`[`Parameter`](../API/parameter.md#Parameter)  

> Creates an Parameter with "number" Lua type with the given default value
> and other optional properties.
### enum(id : [`ParameterId`](#ParameterId), default : [`ParameterEnumDefault`](#ParameterEnumDefault), values : [`string`](../API/builtins/string.md)[], name : [`ParameterName`](#ParameterName)[`?`](../API/builtins/nil.md), description : [`ParameterDescription`](#ParameterDescription)[`?`](../API/builtins/nil.md))<a name="enum"></a>
`->`[`Parameter`](../API/parameter.md#Parameter)  

> Creates an Parameter with a "string" Lua type with the given default value,
> set of valid values to choose from and other optional properties.  



---  
## Aliases  
### ParameterBooleanDefault<a name="ParameterBooleanDefault"></a>
[`boolean`](../API/builtins/boolean.md)  
> Default boolean value.  
  
### ParameterDescription<a name="ParameterDescription"></a>
[`string`](../API/builtins/string.md)  
> Optional long description of the parameter describing what the parameter does.  
  
### ParameterEnumDefault<a name="ParameterEnumDefault"></a>
[`string`](../API/builtins/string.md)  
> Default string value. Must be a valid string within the specified value set.  
  
### ParameterId<a name="ParameterId"></a>
[`string`](../API/builtins/string.md)  
> Unique id of the parameter. The id will be used in the `parameter` context table as key.  
  
### ParameterIntegerDefault<a name="ParameterIntegerDefault"></a>
[`integer`](../API/builtins/integer.md)  
> Default integer value. Must be in the specified value range.  
  
### ParameterIntegerRange<a name="ParameterIntegerRange"></a>
{ 1 : [`integer`](../API/builtins/integer.md), 2 : [`integer`](../API/builtins/integer.md) }  
> Optional value range. When undefined (0.0 - 1.0)  
  
### ParameterName<a name="ParameterName"></a>
[`string`](../API/builtins/string.md)  
> Optional name of the parameter as displayed to the user. When undefined, the id is used.  
  
### ParameterNumberDefault<a name="ParameterNumberDefault"></a>
[`number`](../API/builtins/number.md)  
> Default number value. Must be in the specified value range.  
  
### ParameterNumberRange<a name="ParameterNumberRange"></a>
{ 1 : [`number`](../API/builtins/number.md), 2 : [`number`](../API/builtins/number.md) }  
> Optional value range. When undefined (0 - 100)  
  



