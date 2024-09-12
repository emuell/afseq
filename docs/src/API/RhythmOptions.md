# RhythmOptions  
> Construction options for a new rhythm.  

<!-- toc -->
  

---  
## Properties
### emit : [`Cycle`](../API/Cycle.md) | [`Sequence`](../API/Sequence.md) | [`Note`](../API/Note.md) | [`NoteValue`](#NoteValue) | [`Note`](../API/Note.md) | [`NoteValue`](#NoteValue)[] | (context : [`EmitterContext`](../API/EmitterContext.md)) `->` [`NoteValue`](#NoteValue) | (context : [`EmitterContext`](../API/EmitterContext.md)) `->` (context : [`EmitterContext`](../API/EmitterContext.md)) `->` [`NoteValue`](#NoteValue) {#emit}
> Specify the melodic pattern of the rhythm. For every pulse in the rhythmical pattern, the event
> from the specified emit sequence. When the end of the sequence is reached, it starts again from
> the beginning.<br>
> 
> To generate notes dynamically, you can pass a function or a function iterator, instead of a
> fixed array or sequence of notes.<br>
> 
> Events can also be generated using the tidal cycle mini-notation. Cycles are repeated endlessly
> by default, and have the duration of a single pulse in the pattern. Patterns can be used to
> sequence cycles too.
> 
> #### examples:
> ```lua
> -- a sequence of c4, g4
> emit = {"c4", "g4"}
> -- a chord of c4, d#4, g4
> emit = {{"c4", "d#4", "g4"}} -- or {"c4'min"}
> -- a sequence of c4, g4 with volume 0.5
> emit = sequence{"c4", "g4"}:volume(0.5)
> 
> -- stateless generator function
> -- a function
> emit = function(context)
>   return 48 + math.random(1, 4) * 5
> end
> 
> -- stateful generator function
> emit = function(initial_context)
>   local count, step, notes = 1, 2, scale("c5", "minor").notes
>   ---@param context EmitterContext
>   return function(context)
>     local key = notes[count]
>     count = (count + step - 1) % #notes + 1
>     return { key = key, volume = 0.5 }
>   end
> end
> 
> -- a note pattern
> local tritone = scale("c5", "tritone")
> ...
> emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
>   pattern.from(tritone:chord(5, 4)):euclidean(6)
> 
> -- a tidal cycle
> emit = cycle("<[a3 c4 e4 a4]*3 [d4 g3 g4 c4]>")
> --
> ```

### gate : [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md)[] | (context : [`GateContext`](../API/GateContext.md)) `->` [`boolean`](../API/builtins/boolean.md) | (context : [`GateContext`](../API/GateContext.md)) `->` (context : [`GateContext`](../API/GateContext.md)) `->` [`boolean`](../API/builtins/boolean.md) {#gate}
> Optional pulse train filter function or generator function which filters events between
> the pattern and emitter. By default a threshold gate, which passes all pulse values
> greater than zero. 
> 
> Custom function should returns true when a pattern pulse value should be passed, 
> and false when the emitter should be skipped.  
> 
> #### examples:
> ```lua
> -- probability gate: skips all 0s, passes all 1s. pulse alues in range (0, 1) are
> -- maybe passed, using the pulse value as probablility.
> gate = function(context)
>   return context.pulse_value > math.random()
> end
> ```

### inputs : [`InputParameter`](../API/InputParameter.md)[] {#inputs}
> Define optional input parameters for the rhythm. Input parameters can dynamically
> change a rhythms behavior everywhere where `context`s are passed, e.g. in pattern,
> gate, emitter or cycle map generator functions.
> 
> ## examples:
> ```lua
> -- trigger a single note as specified by input parameter 'note'
> -- when input parameter 'enabled' is true, else triggers nothing.
>   inputs = {
>     parameter.boolean("enabled", true),
>     parameter.integer("note", 48, { 0, 127 })
>   },
> -- [...]
>   emit = function(context)
>     if context.inputs.enabled then -- boolean value
>       return note(context.inputs.note) -- integer value
>     else
>       return nil
>     end
>   end
> ```

### offset : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#offset}
> Optional offset in `unit * resolution` time units. By default 0.
> When set, the rhythm's event output will be delayed by the given offset value.
> #### examples:
> ```lua
> unit = "1/4",
> resolution = 4,
> offset = 4 -- start emitting after 4*4 beats
> ```

### pattern : [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md)[] | (context : [`PatternContext`](../API/PatternContext.md)) `->` [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md) | (context : [`PatternContext`](../API/PatternContext.md)) `->` (context : [`PatternContext`](../API/PatternContext.md)) `->` [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md) {#pattern}
> Specify the rhythmical pattern of the emitter. Each pulse with a value of 1 or true
> will cause an event from the `emitter` property to be triggered in the emitters
> time unit. 0 or nil values never trigger, and values in-between do *maybe* trigger.
> 
> To create deterministic random patterns, seed the random number generator before
> creating the rhythm via `math.randomseed(some_seed)`
> 
> Patterns can contains subdivisions, sub tables of pulses, to "cram" multiple pulses
> into a single pulse's time interval. This way more complex rhythmical patterns can
> be created.
> 
> When no pattern is defined, a constant pulse of `1` is triggered by the rhythm.
> 
> Just like the `emitter` property, patterns can either be a fixed array of values or a
> function or iterator which produces values dynamically.
> 
> #### examples:
> ```lua
> -- a fixed pattern
> pattern = { 1, 0, 0, 1 }
> -- maybe trigger with probabilities
> pattern = { 1, 0, 0.5, 0.9 }
> -- "cram" pulses into a single pulse slot via subdivisions
> pattern = { 1, { 1, 1, 1 } }
> 
> -- fixed patterns created via "patterns"
> pattern = pattern.from{ 1, 0 } * 3 + { 1, 1 }
> pattern = pattern.euclidean(7, 16, 2)
> 
> -- stateless generator function
> pattern = function(context)
>   return math.random(0, 1)
> end
> 
> -- stateful generator function
> pattern = function(context)
>   local my_pattern = table.create({0, 6, 10})
>   ---@param context EmitterContext
>   return function(context)
>     return my_pattern:find((context.step - 1) % 16) ~= nil
>   end
> end
> 
> ```

### repeats : [`boolean`](../API/builtins/boolean.md) | [`integer`](../API/builtins/integer.md) {#repeats}
> If and how many times a pattern should repeat. When 0 or false, the pattern does not repeat
> and plays back only once. When true, the pattern repeats endlessly, which is the default.
> When a number > 0, this specifies the number of times the pattern repeats until it stops.
> 
> Note: When `pattern` is a function or iterator, the repeat count is the number of
> *function calls or iteration steps*. When the pattern is a pulse array, this is the number of
> times the whole pattern gets repeated.
> 
> #### examples:
> ```lua
> repeat = 0 -- one-shot
> repeat = false -- also a one-shot
> repeat = 3 -- play the pattern 4 times
> repeat = true -- play & repeat forever
> ```

### resolution : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#resolution}
> Factor which is applied on `unit` to specify the final time resolution of the emitter.
> #### examples:
> ```lua
> unit = "beats", resolution = 1.01 --> slightly off beat pulse
> unit = "1/16", resolution = 4/3 --> triplet
> ```

### unit : `"ms"` | `"seconds"` | `"bars"` | `"beats"` | `"1/1"` | `"1/2"` | `"1/4"` | `"1/8"` | `"1/16"` | `"1/32"` | `"1/64"` {#unit}
> Base time unit of the emitter. Use `resolution` to apply an additional factor, in order to
> create other less common rhythm bases.
> #### examples:
> ```lua
> unit = "beats", resolution = 1.01 --> slightly off beat pulse
> unit = "1/16", resolution = 4/3 --> triplet
> ```

  

