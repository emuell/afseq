# pattern
<!-- toc -->
# Pattern<a name="Pattern"></a>  
> Array alike table with helper functions to ease creating rhythmic patterns.
> 
> #### examples:
> ```lua
> -- using + and * operators to combine patterns
> pattern.from{ 0, 1 } * 3 + { 1, 0 }
> ```
> ```lua
> -- repeating, spreading and subsets
> pattern.from{ 0, 1, { 1, 1 } }:repeat_n(4):spread(1.25):take(16)
> ```
> ```lua
> -- euclidean patterns
> pattern.euclidean(12, 16)
> pattern.from{ 1, 0.5, 1, 1 }:euclidean(12)
> ```
> ```lua
> -- generate/init from functions
> pattern.new(8):init(1) --> 1,1,1,1,1,1,1,1
> pattern.new(12):init(function() return math.random(0.5, 1.0) end )
> pattern.new(16):init(scale("c", "minor").notes_iter())
> ```
> ```lua
> -- generate note patterns
> pattern.from{ "c4", "g4", "a4" } * 7 + { "a4", "g4", "c4" }
> ```
> ```lua
> -- generate chords from degree values
> pattern.from{ 1, 5, 6, 4 }:map(function(index, degree)
>   return scale("c", "minor"):chord(degree)
> end)
> ```  

---  
## Functions
### new(length : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), value : [`PulseValue`](#PulseValue) | (index : [`integer`](../API/builtins/integer.md)) `->` [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="new"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Create a new empty pattern or pattern with the given length and pulse value.
> 
> 
> #### examples:
> ```lua
> pattern.new(4,1) --> {1,1,1,1}
> pattern.new(4, function() return math.random() end)
> ```
### from(...[`PulseValue`](#PulseValue) | [`PulseValue`](#PulseValue)[])<a name="from"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Create a new pattern from an existing set of values or tables.
> When passing tables, those will be flattened.
> 
> #### examples:
> ```lua
> pattern.from(1,0,1,0) --> {1,0,1,0}
> pattern.from({1,0},{1,0}) --> {1,0,1,0}
> ```
### copy(self : [`Pattern`](../API/pattern.md#Pattern))<a name="copy"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

>  create a shallow-copy of the given pattern (or self)
> 
> #### examples:
> ```lua
> local p = pattern.from(1, 0)
> local p2 = p:copy() --> {1,0}
> ```
### distributed(steps : [`integer`](../API/builtins/integer.md) | [`table`](../API/builtins/table.md), length : [`integer`](../API/builtins/integer.md), offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="distributed"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Create an new pattern or spread and existing pattern evenly within the given length.
> Similar, but not exactly like `euclidean`.
> 
> Shortcut for `pattern.from{1,1,1}:spread(length / #self):rotate(offset)`
> 
> #### examples:
> ```lua
> pattern.distributed(3, 8) --> {1,0,0,1,0,1,0}
> pattern.from{1,1}:distributed(4, 1) --> {0,1,0,1}
> ```
### euclidean(steps : [`integer`](../API/builtins/integer.md) | [`table`](../API/builtins/table.md), length : [`integer`](../API/builtins/integer.md), offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="euclidean"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Create a new euclidean rhythm pattern with the given pulses or number of new pulses
> in the given length and optionally rotate the contents.
> [Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
> 
> #### examples:
> ```lua
> pattern.euclidean(3, 8)
>  --> {1,0,0,1,0,0,1,0}
> pattern.from{"a", "b", "c"}:euclidean(8, 0, "-")
>  --> {"a","-","-","b","-","-","c","-"}
> ```
### unpack(self : [`Pattern`](../API/pattern.md#Pattern))<a name="unpack"></a>
`->`... : [`PulseValue`](#PulseValue)  

> Shortcut for table.unpack(pattern): returns elements from this pattern as var args.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,2,3,4}
> local v1, v2, v3, v4 = p:unpack()
> ```
### subrange(self : [`Pattern`](../API/pattern.md#Pattern), i : [`integer`](../API/builtins/integer.md), j : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="subrange"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Get sub range from the pattern as new pattern.
> When the given length is past end of this pattern its filled up with empty values.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,2,3,4}
> p = p:subrange(2,3) --> {2,3}
> p = p:subrange(1,4,"X") --> {2,3,"X","X"}
> ```
### take(self : [`Pattern`](../API/pattern.md#Pattern), length : [`integer`](../API/builtins/integer.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="take"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Get first n items from the pattern as new pattern.
> When the given length is past end of this pattern its filled up with empty values.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,2,3,4}
> p = p:take(2) --> {1,2}
> p = p:take(4, "") --> {1,2,"",""}
> ```
### clear(self : [`Pattern`](../API/pattern.md#Pattern))<a name="clear"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Clear a pattern, remove all its contents.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,0}
> p:clear() --> {}
> ```
### init(self : [`Pattern`](../API/pattern.md#Pattern), value : [`PulseValue`](#PulseValue) | (index : [`integer`](../API/builtins/integer.md)) `->` [`PulseValue`](#PulseValue), length : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md))<a name="init"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Fill pattern with the given value or generator function in length.
> 
> #### examples:
> ```lua
> local p = pattern.from{0,0}
> p:init(1) --> {1,1}
> p:init("X", 3) --> {"X","X", "X"}
> ```
### map(self : [`Pattern`](../API/pattern.md#Pattern), fun : (index : [`integer`](../API/builtins/integer.md), value : [`PulseValue`](#PulseValue)) `->` [`PulseValue`](#PulseValue))<a name="map"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Apply the given function to every item in the pattern.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,3,5}
> p:map(function(k, v)
>   return scale("c", "minor"):degree(v)
> end) --> {48, 51, 55}
> ```
### reverse(self : [`Pattern`](../API/pattern.md#Pattern))<a name="reverse"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Invert the order of items.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,2,3}
> p:reverse() --> {3,2,1}
> ```
### rotate(self : [`Pattern`](../API/pattern.md#Pattern), amount : [`integer`](../API/builtins/integer.md))<a name="rotate"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Shift contents by the given amount to the left (negative amount) or right.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,0,0}
> p:rotate(1) --> {0,1,0}
> p:rotate(-2) --> {0,0,1}
> ```
### push_back(self : [`Pattern`](../API/pattern.md#Pattern), ...[`PulseValue`](#PulseValue)[] | [`PulseValue`](#PulseValue))<a name="push_back"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Push a single or multiple number of items or other pattern contents to the end of the pattern.
> Note: When passing array alike tables or patterns, they will be *unpacked*.
> 
> #### examples:
> ```lua
> local p = pattern.new()
> p:push_back(1) --> {1}
> p:push_back(2,3) --> {1,2,3}
> p:push_back{4} --> {1,2,3,4}
> p:push_back({5,{6,7}) --> {1,2,3,4,5,6,7}
> ```
### pop_back(self : [`Pattern`](../API/pattern.md#Pattern))<a name="pop_back"></a>
`->`[`PulseValue`](#PulseValue)  

> Remove an entry from the back of the pattern. returns the popped item.
> 
> #### examples:
> ```lua
> local p = pattern.from({1,2})
> p:pop_back() --> {1}
> p:pop_back() --> {}
> p:pop_back() --> {}
> ```
### repeat_n(self : [`Pattern`](../API/pattern.md#Pattern), count : [`integer`](../API/builtins/integer.md))<a name="repeat_n"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Duplicate the pattern n times.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,2,3}
> patterns:repeat_n(2) --> {1,2,3,1,2,3}
> ```
### spread(self : [`Pattern`](../API/pattern.md#Pattern), amount : [`number`](../API/builtins/number.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md))<a name="spread"></a>
`->`[`Pattern`](../API/pattern.md#Pattern)  

> Expand (with amount > 1) or shrink (amount < 1) the length of the pattern by the
> given factor, spreading allowed content evenly and filling gaps with 0 or the
> given empty value.
> 
> #### examples:
> ```lua
> local p = pattern.from{1,1}
> p:spread(2) --> {1,0,1,0}
> p:spread(1/2) --> {1,1}
> ```
### tostring(self : [`Pattern`](../API/pattern.md#Pattern))<a name="tostring"></a>
`->`[`string`](../API/builtins/string.md)  

> Serialze a pattern for display/debugging purposes.
> 
> #### examples:
> ```lua
> pattern.euclidean(3, 8):tostring() --> "{1, 1, 1, 0}"
> ```  



---  
## Aliases  
### PulseValue<a name="PulseValue"></a>
[`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`table`](../API/builtins/table.md)  
> Valid pulse value in a pattern  
  



