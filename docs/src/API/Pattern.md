# Pattern  
> Array alike table with helper functions to ease creating rhythmic patterns.
> 
> #### examples:
> ```lua
> -- using + and * operators to combine patterns
> pattern.from{ 0, 1 } * 3 + { 1, 0 }
> -- repeating, spreading and subsets
> pattern.from{ 0, 1, { 1, 1 } }:repeat_n(4):spread(1.25):take(16)
> -- euclidean patterns
> pattern.euclidean(12, 16)
> pattern.from{ 1, 0.5, 1, 1 }:euclidean(12)
> -- generate/init from functions
> pattern.new(12):init(function() return math.random(0.5, 1.0) end )
> -- generate note patterns
> pattern.from{ "c4", "g4", "a4" } * 7 + { "a4", "g4", "c4" }
> pattern.from{ 1, 5, 6, 4 }:map(function(index, degree)
>   return scale("c", "minor"):chord(degree)
> end)
> ```  

<!-- toc -->
  

---  
## Functions
### clear(self : [`Pattern`](../API/Pattern.md)) {#clear}
`->`[`Pattern`](../API/Pattern.md)  

> Clear a pattern, remove all its contents.
### copy(self : [`Pattern`](../API/Pattern.md)) {#copy}
`->`[`Pattern`](../API/Pattern.md)  

>  create a shallow-copy of the given pattern (or self)
### distributed(steps : [`integer`](../API/builtins/integer.md) | [`table`](../API/builtins/table.md), length : [`integer`](../API/builtins/integer.md), offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md)) {#distributed}
`->`[`Pattern`](../API/Pattern.md)  

> Create an new pattern or spread and existing pattern evenly within the given length.
> Similar, but not exactly like "euclidean".
> 
> Shortcut for:
> ```lua
> pattern.new(1, steps):spread(length / steps):rotate(offset) -- or
> pattern.from{1,1,1}:spread(length / #self):rotate(offset)
> ```
### euclidean(steps : [`integer`](../API/builtins/integer.md) | [`table`](../API/builtins/table.md), length : [`integer`](../API/builtins/integer.md), offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md)) {#euclidean}
`->`[`Pattern`](../API/Pattern.md)  

> Create a new euclidean rhythm pattern with the given pulses or number of new pulses
> in the given length and optionally rotate the contents.
> [Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
### from(...[`PulseValue`](#PulseValue) | [`PulseValue`](#PulseValue)[]) {#from}
`->`[`Pattern`](../API/Pattern.md)  

> Create a new pattern from a set of values or tables.
> When passing tables, those will be flattened.
### init(self : [`Pattern`](../API/Pattern.md), value : [`PulseValue`](#PulseValue) | (index : [`integer`](../API/builtins/integer.md)) `->` [`PulseValue`](#PulseValue), length : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md)) {#init}
`->`[`Pattern`](../API/Pattern.md)  

> Fill pattern with the given value or generator function in length.
### map(self : [`Pattern`](../API/Pattern.md), fun : (index : [`integer`](../API/builtins/integer.md), value : [`PulseValue`](#PulseValue)) `->` [`PulseValue`](#PulseValue)) {#map}
`->`[`Pattern`](../API/Pattern.md)  

> Apply the given function to every item in the pattern.
### new(length : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md)) {#new}
`->`[`Pattern`](../API/Pattern.md)  

> Create a new empty pattern or pattern with the given length.
### pop(self : [`Pattern`](../API/Pattern.md)) {#pop}
`->`[`PulseValue`](#PulseValue)  

> Remove an entry from the back of the pattern and returns the popped item.
### push(self : [`Pattern`](../API/Pattern.md), ...[`PulseValue`](#PulseValue)[] | [`PulseValue`](#PulseValue)) {#push}
`->`[`Pattern`](../API/Pattern.md)  

> Push any number of items or other pattern contents to the end of the pattern.
> When passing array alike tables or patterns, they will be unpacked.
### repeat_n(self : [`Pattern`](../API/Pattern.md), count : [`integer`](../API/builtins/integer.md)) {#repeat_n}
`->`[`Pattern`](../API/Pattern.md)  

> Duplicate the pattern n times.
### reverse(self : [`Pattern`](../API/Pattern.md)) {#reverse}
`->`[`Pattern`](../API/Pattern.md)  

> Invert the order of items.
### rotate(self : [`Pattern`](../API/Pattern.md), amount : [`integer`](../API/builtins/integer.md)) {#rotate}
`->`[`Pattern`](../API/Pattern.md)  

> Shift contents by the given amount to the left (negative amount) or right.
### spread(self : [`Pattern`](../API/Pattern.md), amount : [`number`](../API/builtins/number.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md)) {#spread}
`->`[`Pattern`](../API/Pattern.md)  

> Expand (with amount > 1) or shrink (amount < 1) the length of the pattern by the
> given factor, spreading allowed content evenly and filling gaps with 0 or the
> given empty value.
### subrange(self : [`Pattern`](../API/Pattern.md), i : [`integer`](../API/builtins/integer.md), j : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md), empty_value : [`PulseValue`](#PulseValue)[`?`](../API/builtins/nil.md)) {#subrange}
`->`[`Pattern`](../API/Pattern.md)  

> Get sub range from the pattern as new pattern.
> When the given length is past end of this pattern its filled up with empty values.
### take(self : [`Pattern`](../API/Pattern.md), length : [`integer`](../API/builtins/integer.md)) {#take}
`->`[`Pattern`](../API/Pattern.md)  

> Get first n items from the pattern as new pattern.
### unpack(self : [`Pattern`](../API/Pattern.md)) {#unpack}
`->`[`PulseValue`](#PulseValue)[]  

> Shortcut for table.unpack(pattern): returns elements from this pattern as var args.  

