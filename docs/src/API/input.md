# input
<!-- toc -->
# InputParameter<a name="InputParameter"></a>  
> Opaque input parameter user data. Construct new input parameters via the `parameter.XXX(...)`
> functions.  



# Parameter<a name="Parameter"></a>  
> Contains functions to construct new input parameters. Input parameter values can be accessed
> via functionn `contexts` in pattern, gate and emitter functions or generators.  

---  
## Functions
### boolean(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterBooleanDefault`](#InputParameterBooleanDefault), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md))<a name="boolean"></a>
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "boolean" Lua type with the given default value
> and other optional properties.
### integer(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterIntegerDefault`](#InputParameterIntegerDefault), range : [`InputParameterIntegerRange`](#InputParameterIntegerRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md))<a name="integer"></a>
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "integer" Lua type with the given default value
> and other optional properties.
### number(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterNumberDefault`](#InputParameterNumberDefault), range : [`InputParameterNumberRange`](#InputParameterNumberRange)[`?`](../API/builtins/nil.md), name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md))<a name="number"></a>
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with "number" Lua type with the given default value
> and other optional properties.
### enum(id : [`InputParameterId`](#InputParameterId), default : [`InputParameterEnumDefault`](#InputParameterEnumDefault), values : [`string`](../API/builtins/string.md)[], name : [`InputParameterName`](#InputParameterName)[`?`](../API/builtins/nil.md), description : [`InputParameterDescription`](#InputParameterDescription)[`?`](../API/builtins/nil.md))<a name="enum"></a>
`->`[`InputParameter`](../API/input.md#InputParameter)  

> Creates an InputParameter with a "string" Lua type with the given default value,
> set of valid values to choose from and other optional properties.  



---  
## Aliases  
### InputParameterBooleanDefault<a name="InputParameterBooleanDefault"></a>
[`boolean`](../API/builtins/boolean.md)  
> Default boolean value.  
  
### InputParameterDescription<a name="InputParameterDescription"></a>
[`string`](../API/builtins/string.md)  
> Optional long description of the parameter describing what the parameter does.  
  
### InputParameterEnumDefault<a name="InputParameterEnumDefault"></a>
[`string`](../API/builtins/string.md)  
> Default string value. Must be a valid string within the specified value set.  
  
### InputParameterId<a name="InputParameterId"></a>
[`string`](../API/builtins/string.md)  
> Unique id of the parameter. The id will be used in the `input` context table as key.  
  
### InputParameterIntegerDefault<a name="InputParameterIntegerDefault"></a>
[`integer`](../API/builtins/integer.md)  
> Default integer value. Must be in the specified value range.  
  
### InputParameterIntegerRange<a name="InputParameterIntegerRange"></a>
{ 1 : [`integer`](../API/builtins/integer.md), 2 : [`integer`](../API/builtins/integer.md) }  
> Optional value range. When undefined (0.0 - 1.0)  
  
### InputParameterName<a name="InputParameterName"></a>
[`string`](../API/builtins/string.md)  
> Optional name of the parameter as displayed to the user. When undefined, the id is used.  
  
### InputParameterNumberDefault<a name="InputParameterNumberDefault"></a>
[`number`](../API/builtins/number.md)  
> Default number value. Must be in the specified value range.  
  
### InputParameterNumberRange<a name="InputParameterNumberRange"></a>
{ 1 : [`number`](../API/builtins/number.md), 2 : [`number`](../API/builtins/number.md) }  
> Optional value range. When undefined (0 - 100)  
  



