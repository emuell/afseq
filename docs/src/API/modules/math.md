# math<a name="math"></a>  

---  
## Functions
### imod(index : [`integer`](../../API/builtins/integer.md), length : [`integer`](../../API/builtins/integer.md))<a name="imod"></a>
`->`[`integer`](../../API/builtins/integer.md)  

> Wrap a lua 1 based integer index into the given array/table length.
> 
> -> `(index - 1) % length + 1`
### random(m : [`integer`](../../API/builtins/integer.md), n : [`integer`](../../API/builtins/integer.md))<a name="random"></a>
`->`[`integer`](../../API/builtins/integer.md)  

> * `math.random()`: Returns a float in the range [0,1).
> * `math.random(n)`: Returns a integer in the range [1, n].
> * `math.random(m, n)`: Returns a integer in the range [m, n].
> 
> 
> [View documents](http://www.lua.org/manual/5.4/manual.html#pdf-math.random)
### randomseed(x : [`integer`](../../API/builtins/integer.md))<a name="randomseed"></a>
> * `math.randomseed(x, y)`: Concatenate `x` and `y` into a 128-bit `seed` to reinitialize the pseudo-random generator.
> * `math.randomseed(x)`: Equate to `math.randomseed(x, 0)` .
> * `math.randomseed()`: Generates a seed with a weak attempt for randomness.
> 
> 
> [View documents](http://www.lua.org/manual/5.4/manual.html#pdf-math.randomseed)
### randomstate(seed : [`integer`](../../API/builtins/integer.md)[`?`](../../API/builtins/nil.md))<a name="randomstate"></a>
`->`(m : [`integer`](../../API/builtins/integer.md)[`?`](../../API/builtins/nil.md), n : [`integer`](../../API/builtins/integer.md)[`?`](../../API/builtins/nil.md)) `->` [`number`](../../API/builtins/number.md)  

> Create a new local random number state with the given optional seed value.
> 
> When no seed value is specified, the global `math.randomseed` value is used.
> When no global seed value is available, a new unique random seed is created.
> 
> Random states can be useful to create multiple, separate seeded random number
> generators, e.g. in pattern, gate or emit generators, which get reset with the
> generator functions.
> 
> #### examples:
> 
> ```lua
> return pattern {
>   event = function(init_context)
>     -- use a unique random sequence every time the pattern gets (re)triggered
>     local rand = math.randomstate(12345)
>     return function(context)
>       if rand(1, 10) > 5 then
>         return "c5"
>       else
>         return "g4"
>       end
>   end
> }
> ```  

