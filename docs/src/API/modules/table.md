# table {#table}  

---  
## Functions
### new(t : [`table`](../../API/builtins/table.md)[`?`](../../API/builtins/nil.md)) {#new}
`->`[`table`](../../API/builtins/table.md) | tablelib  

> Create a new empty table, or convert an exiting table to an object that uses the global
> 'table.XXX' functions as methods, just like strings in Lua do.
> 
> #### examples:
> ```lua
> t = table.new(); t:insert("a"); print(t[1]) -> "a";
> t = table.new{1,2,3}; print(t:concat("|")); -> "1|2|3";
> ```
### contains(t : [`table`](../../API/builtins/table.md), value : [`any`](../../API/builtins/any.md), start_index : [`integer`](../../API/builtins/integer.md)[`?`](../../API/builtins/nil.md)) {#contains}
`->`[`boolean`](../../API/builtins/boolean.md)  

> Test if the table contains an entry matching the given value,
> starting from element number start_index or 1.
> 
> #### examples:
> ```lua
> t = {"a", "b"}; table.contains(t, "a") --> true
> t = {a=1, b=2}; table.contains(t, 2) --> true
> t = {"a", "b"}; table.contains(t, "c") --> false
> ```
### find(t : [`table`](../../API/builtins/table.md), value : [`any`](../../API/builtins/any.md), start_index : [`integer`](../../API/builtins/integer.md)[`?`](../../API/builtins/nil.md)) {#find}
`->`key : [`any`](../../API/builtins/any.md)  

> Find first match of given value, starting from element
>  number start_index or 1.
> 
> Returns the first *key* that matches the value or nil
> 
> #### examples:
> ```lua
> t = {"a", "b"}; table.find(t, "a") --> 1
> t = {a=1, b=2}; table.find(t, 2) --> "b"
> t = {"a", "b", "a"}; table.find(t, "a", 2) --> "3"
> t = {"a", "b"}; table.find(t, "c") --> nil
> ```
### tostring(t : [`table`](../../API/builtins/table.md)) {#tostring}
`->`[`string`](../../API/builtins/string.md)  

> Serialze a table to a string for display/debugging purposes.
### copy(t : [`table`](../../API/builtins/table.md)) {#copy}
`->`[`table`](../../API/builtins/table.md)  

> Copy the metatable and all elements non recursively into a new table.
> Creates a clone with shared references.  

