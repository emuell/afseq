
<!-- toc -->

# Global {#Global}  

---  
## Functions
### rhythm(options : [`RhythmOptions`](../API/rhythm.md#RhythmOptions)) {#rhythm}
`->`[`userdata`](../API/builtins/userdata.md)  

> Create a new rhythm with the given configuration.
> 
> #### examples:
> ```lua
> -- trigger a chord sequence every 4 bars after 4 bars
> return rhythm {
>   unit = "bars",
>   resolution = 4,
>   offset = 1,
>   emit = sequence("c4'm", note("g3'm7"):transpose({0, 12, 0, 0}))
> }
> 
> --trigger notes in an euclidean triplet pattern
> return rhythm {
>   unit = "1/8",
>   resolution = 3/2,
>   pattern = pattern.euclidean(6, 16, 2),
>   emit = sequence("c3", "c3", note{ "c4", "a4" }:volume(0.75))
> }
> 
> --trigger notes in a seeded, random subdivision pattern
> math.randomseed(23498)
> return rhythm {
>   unit = "1/8",
>   pattern = { 1, { 0, 1 }, 0, 0.3, 0.2, 1, { 0.5, 0.1, 1 }, 0.5 },
>   emit = { "c4" },
> }
> 
> --trigger random notes in a random pattern from a pentatonic scale
> return rhythm {
>   unit = "1/16",
>   pattern = function(context)
>     return (context.pulse_step % 4 == 1) or (math.random() > 0.8)
>   end,
>   emit = function(_init_context)
>     local cmin = scale("c5", "pentatonic minor").notes
>     return function(_context)
>       return { key = cmin[math.random(#cmin)], volume = 0.7 }
>     end
>   end
> }
> 
> --play a seeded tidal cycle
> math.randomseed(9347565)
> return rhythm {
>   unit = "bars", -- emit one cycle per bar
>   emit = cycle("[c4 [f5 f4]*2]|[c4 [g5 g4]*3]")
> }
> --
> ```  



---  
## Aliases  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  
### Pulse {#Pulse}
[`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md)[] | `0` | `1` | [`nil`](../API/builtins/nil.md)  
> ```lua
> -- Single pulse value or a nested subdivision of pulses within a pattern.
> Pulse:
>     | 0
>     | 1
> ```  
  



# RhythmOptions {#RhythmOptions}  
> Construction options for a new rhythm.  

---  
## Properties
### unit : `"ms"` | `"seconds"` | `"bars"` | `"beats"` | `"1/1"` | `"1/2"` | `"1/4"` | `"1/8"` | `"1/16"` | `"1/32"` | `"1/64"` {#unit}
> Base time unit of the emitter. Use `resolution` to apply an additional factor, in order to
> create other less common rhythm bases.
> #### examples:
> ```lua
> unit = "beats", resolution = 1.01 --> slightly off beat pulse
> unit = "1/16", resolution = 4/3 --> triplet
> ```

### resolution : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#resolution}
> Factor which is applied on `unit` to specify the final time resolution of the emitter.
> #### examples:
> ```lua
> unit = "beats", resolution = 1.01 --> slightly off beat pulse
> unit = "1/16", resolution = 4/3 --> triplet
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

### inputs : [`InputParameter`](../API/input.md#InputParameter)[] {#inputs}
> Define optional input parameters for the rhythm. Input parameters can dynamically
> change a rhythms behavior everywhere where `context`s are passed, e.g. in pattern,
> gate, emitter or cycle map generator functions.
> 
> #### examples:
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

### pattern : [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md)[] | (context : [`PatternContext`](../API/rhythm.md#PatternContext)) `->` [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md) | (context : [`PatternContext`](../API/rhythm.md#PatternContext)) `->` (context : [`PatternContext`](../API/rhythm.md#PatternContext)) `->` [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md) {#pattern}
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
> Just like the `emitter` property, patterns can either be a static array of values
> or a function or generators which produces values dynamically.
> 
> #### examples:
> ```lua
> -- static pattern
> pattern = { 1, 0, 0, 1 }
> -- "cram" pulses into a single pulse slot via subdivisions
> pattern = { 1, { 1, 1, 1 } }
> 
> -- patterns created via the "patterns" lib
> pattern = pattern.from{ 1, 0 } * 3 + { 1, 1 }
> pattern = pattern.euclidean(7, 16, 2)
> 
> -- stateless pattern function
> pattern = function(_context)
>   return math.random(0, 1)
> end
> 
> -- stateful generator function
> pattern = function(_init_context)
>   local triggers = table.new{ 0, 6, 10 }
>   return function(context)
>     local step = (context.step - 1) % 16
>     return triggers:contains(step)
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

### gate : (context : [`GateContext`](../API/rhythm.md#GateContext)) `->` [`boolean`](../API/builtins/boolean.md) | (context : [`GateContext`](../API/rhythm.md#GateContext)) `->` (context : [`GateContext`](../API/rhythm.md#GateContext)) `->` [`boolean`](../API/builtins/boolean.md) {#gate}
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

### emit : [`Cycle`](../API/cycle.md#Cycle) | [`Sequence`](../API/sequence.md#Sequence) | [`Note`](../API/note.md#Note) | [`NoteValue`](#NoteValue) | [`Note`](../API/note.md#Note) | [`NoteValue`](#NoteValue)[] | (context : [`EmitterContext`](../API/rhythm.md#EmitterContext)) `->` [`NoteValue`](#NoteValue) | (context : [`EmitterContext`](../API/rhythm.md#EmitterContext)) `->` (context : [`EmitterContext`](../API/rhythm.md#EmitterContext)) `->` [`NoteValue`](#NoteValue) {#emit}
> Specify the melodic pattern of the rhythm. For every pulse in the rhythmical pattern, the event
> from the specified emit sequence. When the end of the sequence is reached, it starts again from
> the beginning.
> 
> To generate notes dynamically, you can pass a function or a function iterator, instead of a
> static array or sequence of notes.
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
> emit = function(_context)
>   return 48 + math.random(1, 4) * 5
> end
> 
> -- stateful generator function
> emit = function(_init_context)
>   local count, step, notes = 1, 2, scale("c5", "minor").notes
>   return function(_context)
>     local key = notes[count]
>     count = (count + step - 1) % #notes + 1
>     return { key = key, volume = 0.5 }
>   end
> end
> 
> -- a note pattern
> local tritone = scale("c5", "tritone")
> .. -- instrument #1,5,7 will be set as specified.
> emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
>   pattern.from(tritone:chord(5, 4)):euclidean(6)
> 
> -- a tidal cycle
> emit = cycle("<[a3 c4 e4 a4]*3 [d4 g3 g4 c4]>")
> --
> ```

  



---  
## Aliases  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  
### Pulse {#Pulse}
[`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | [`boolean`](../API/builtins/boolean.md) | [`number`](../API/builtins/number.md) | `0` | `1` | [`Pulse`](#Pulse) | [`nil`](../API/builtins/nil.md)[] | `0` | `1` | [`nil`](../API/builtins/nil.md)  
> ```lua
> -- Single pulse value or a nested subdivision of pulses within a pattern.
> Pulse:
>     | 0
>     | 1
> ```  
  



# EmitterContext {#EmitterContext}  
> Context passed to 'emit' functions.  

---  
## Properties
### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> Project's tempo in beats per minutes.

### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> Project's beats per bar setting.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> Project's sample rate in samples per second.

### pulse_step : [`integer`](../API/builtins/integer.md) {#pulse_step}
> Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
> Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
> not emit events. Starts from 1 when the rhythm starts running or is reset.

### pulse_time_step : [`number`](../API/builtins/number.md) {#pulse_time_step}
> Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
> Starts from 0 and increases with each new pulse by the pulse's step time duration.

### pulse_time : [`number`](../API/builtins/number.md) {#pulse_time}
> Current pulse's step time as fraction of a full step in the pattern. For simple pulses this
> will be 1, for pulses in subdivisions this will be the reciprocal of the number of steps in the
> subdivision, relative to the parent subdivisions pulse step time.
> #### examples:
> ```lua
> {1, {1, 1}} --> step times: {1, {0.5, 0.5}}
> ```

### pulse_value : [`number`](../API/builtins/number.md) {#pulse_value}
> Current pulse value. For binary pulses this will be 1, 0 pulse values will not cause the emitter
> to be called, so they never end up here.
> Values between 0 and 1 will be used as probabilities and thus are maybe emitted or skipped.

### playback : [`PlaybackState`](#PlaybackState) {#playback}
> Specifies how the emitter currently is running.

### step : [`integer`](../API/builtins/integer.md) {#step}
> Continues step counter, incrementing with each new *emitted* pulse.
> Unlike `pulse_step` this does not include skipped, zero values pulses so it basically counts
> how often the emit function already got called.
> Starts from 1 when the rhythm starts running or is reset.

  



---  
## Aliases  
### PlaybackState {#PlaybackState}
`"running"` | `"seeking"`  
> ```lua
> -- - *seeking*: The emitter is auto-seeked to a target time. All results are discarded. Avoid
> --   unnecessary computations while seeking, and only maintain your generator's internal state.
> -- - *running*: The emitter is played back regularly. Results are audible.
> PlaybackState:
>     | "seeking"
>     | "running"
> ```  
  



# GateContext {#GateContext}  
> Context passed to `gate` functions.  

---  
## Properties
### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> Project's tempo in beats per minutes.

### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> Project's beats per bar setting.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> Project's sample rate in samples per second.

### pulse_step : [`integer`](../API/builtins/integer.md) {#pulse_step}
> Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
> Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
> not emit events. Starts from 1 when the rhythm starts running or is reset.

### pulse_time_step : [`number`](../API/builtins/number.md) {#pulse_time_step}
> Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
> Starts from 0 and increases with each new pulse by the pulse's step time duration.

### pulse_time : [`number`](../API/builtins/number.md) {#pulse_time}
> Current pulse's step time as fraction of a full step in the pattern. For simple pulses this
> will be 1, for pulses in subdivisions this will be the reciprocal of the number of steps in the
> subdivision, relative to the parent subdivisions pulse step time.
> #### examples:
> ```lua
> {1, {1, 1}} --> step times: {1, {0.5, 0.5}}
> ```

### pulse_value : [`number`](../API/builtins/number.md) {#pulse_value}
> Current pulse value. For binary pulses this will be 1, 0 pulse values will not cause the emitter
> to be called, so they never end up here.
> Values between 0 and 1 will be used as probabilities and thus are maybe emitted or skipped.

  



# PatternContext {#PatternContext}  
> Context passed to `pattern` and `gate` functions.  

---  
## Properties
### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> Project's tempo in beats per minutes.

### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> Project's beats per bar setting.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> Project's sample rate in samples per second.

### pulse_step : [`integer`](../API/builtins/integer.md) {#pulse_step}
> Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
> Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
> not emit events. Starts from 1 when the rhythm starts running or is reset.

### pulse_time_step : [`number`](../API/builtins/number.md) {#pulse_time_step}
> Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
> Starts from 0 and increases with each new pulse by the pulse's step time duration.

  



