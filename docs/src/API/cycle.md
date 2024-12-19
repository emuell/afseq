
<!-- toc -->

# Global {#Global}  

---  
## Functions
### cycle(input : [`string`](../API/builtins/string.md)) {#cycle}
`->`[`Cycle`](../API/cycle.md#Cycle)  

>  Create a note sequence from a Tidal Cycles mini-notation string.
> 
>  `cycle` accepts a mini-notation as used by Tidal Cycles, with the following differences:
>  * Stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)
>  * Operators currently only accept numbers on the right side (`a3*2` is valid, `a3*<1 2>` is not)
>  * `:` - Sets the instrument or remappable target instead of selecting samples
>  [Tidal Cycles Reference](https://tidalcycles.org/docs/reference/mini_notation/)
> 
> #### examples:
>  ```lua
> --A chord sequence
> cycle("[c4, e4, g4] [e4, g4, b4] [g4, b4, d5] [b4, d5, f#5]")
> --Arpeggio pattern with variations
> cycle("<c4 e4 g4> <e4 g4> <g4 b4 d5> <b4 f5>")
> --Euclidean Rhythms
> cycle("c4(3,8) e4(5,8) g4(7,8)")
> --Polyrhythm
> cycle("{c4 e4 g4 b4}%2, {f4 d4 a4}%4")
> --Map custom identifiers to notes
> cycle("bd(3,8)"):map({ bd = "c4 #1" })
>  ```  



---  
## Aliases  
### CycleMapFunction {#CycleMapFunction}
(context : [`CycleMapContext`](../API/cycle.md#CycleMapContext), value : [`string`](../API/builtins/string.md)) `->` [`CycleMapNoteValue`](#CycleMapNoteValue)  
  
  
### CycleMapGenerator {#CycleMapGenerator}
(context : [`CycleMapContext`](../API/cycle.md#CycleMapContext), value : [`string`](../API/builtins/string.md)) `->` [`CycleMapFunction`](#CycleMapFunction)  
  
  
### CycleMapNoteValue {#CycleMapNoteValue}
[`Note`](../API/note.md#Note) | [`NoteValue`](#NoteValue) | [`NoteValue`](#NoteValue)[]  
  
  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  
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
  



# Cycle {#Cycle}  

---  
## Functions
### map([*self*](../API/builtins/self.md), map : [`CycleMapFunction`](#CycleMapFunction) | [`CycleMapGenerator`](#CycleMapGenerator) | {  }) {#map}
`->`[`Cycle`](../API/cycle.md#Cycle)  

> Map names in in the cycle to custom note events.
> 
> By default, strings in cycles are interpreted as notes, and integer values as MIDI note
> values. Custom identifiers such as "bd" are undefined and will result into a rest, when
> they are not mapped explicitly.
> 
> #### examples:
> ```lua
> --Using a static map table
> cycle("bd [bd, sn]"):map({
>   bd = "c4",
>   sn = "e4 #1 v0.2"
> })
> --Using a static map table with targets
> cycle("bd:1 <bd:5, bd:7>"):map({
>   -- instrument #1,5,7 will be set as specified
>   bd = { key = "c4", volume = 0.5 },
> })
> --Using a dynamic map function
> cycle("4 5 4 <5 [4|6]>"):map(function(context, value)
>   -- emit a random note with 'value' as octave
>   return math.random(0, 11) + value * 12
> end)
> --Using a dynamic map function generator
> cycle("4 5 4 <4 [5|7]>"):map(function(_init_context)
>   local notes = scale("c", "minor").notes
>   return function(context, value)
>     -- emit a 'cmin' note arp with 'value' as octave
>     local note = notes[math.imod(context.step, #notes)]
>     local octave = tonumber(value)
>     return { key = note + octave * 12 }
>   end
> end)
> --Using a dynamic map function to map values to chord degrees
> cycle("1 5 1 [6|7]"):map(function(_init_context)
>   local cmin = scale("c", "minor")
>   return function(_context, value)
>     return note(cmin:chord(tonumber(value)))
>   end
> end)
> ```  



---  
## Aliases  
### CycleMapFunction {#CycleMapFunction}
(context : [`CycleMapContext`](../API/cycle.md#CycleMapContext), value : [`string`](../API/builtins/string.md)) `->` [`CycleMapNoteValue`](#CycleMapNoteValue)  
  
  
### CycleMapGenerator {#CycleMapGenerator}
(context : [`CycleMapContext`](../API/cycle.md#CycleMapContext), value : [`string`](../API/builtins/string.md)) `->` [`CycleMapFunction`](#CycleMapFunction)  
  
  
### CycleMapNoteValue {#CycleMapNoteValue}
[`Note`](../API/note.md#Note) | [`NoteValue`](#NoteValue) | [`NoteValue`](#NoteValue)[]  
  
  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  
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
  



# CycleMapContext {#CycleMapContext}  
> Context passed to 'cycle:map` functions.  

---  
## Properties
### playback : [`PlaybackState`](#PlaybackState) {#playback}
> Specifies how the cycle currently is running.

### trigger_note : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_note}
> Note value that triggered, started the rhythm, if any.

### channel : [`integer`](../API/builtins/integer.md) {#channel}
> channel/voice index within the cycle. each channel in the cycle gets emitted and thus mapped
> separately, starting with the first channel index 1.

### trigger_volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#trigger_volume}
> Note volume that triggered, started the rhythm, if any.

### step : [`integer`](../API/builtins/integer.md) {#step}
> Continues step counter for each channel, incrementing with each new mapped value in the cycle.
> Starts from 1 when the cycle starts running or after it got reset.

### trigger_offset : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md) {#trigger_offset}
> Note slice offset value that triggered, started the rhythm, if any.

### step_length : [`number`](../API/builtins/number.md) {#step_length}
> step length fraction within the cycle, where 1 is the total duration of a single cycle run.

### inputs : table<[`string`](../API/builtins/string.md), [`boolean`](../API/builtins/boolean.md) | [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md)> {#inputs}
> Current input parameter values, using parameter ids as keys
> and the actual parameter value as value.

### beats_per_min : [`number`](../API/builtins/number.md) {#beats_per_min}
> Project's tempo in beats per minutes.

### beats_per_bar : [`integer`](../API/builtins/integer.md) {#beats_per_bar}
> Project's beats per bar setting.

### samples_per_sec : [`integer`](../API/builtins/integer.md) {#samples_per_sec}
> Project's sample rate in samples per second.

  



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
  



