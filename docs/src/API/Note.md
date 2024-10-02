
<!-- toc -->

# Global {#Global}  

---  
## Functions
### note(...[`NoteValue`](#NoteValue)) {#note}
`->`[`Note`](../API/note.md#Note)  

>  Create a new monophonic or polyphonic note (a chord) from a number value,
>  a note string, chord string or array of note values.
> 
>  In note strings the following prefixes are used to specify optional note
>  attributes:
> ```md
>  -'#' -> instrument (integer > 0)
>  -'v' -> volume (float in range [0-1])
>  -'p' -> panning (float in range [-1-1])
>  -'d' -> delay (float in range [0-1])
> ```
> 
> #### examples:
>  ```lua
>  note(60) -- middle C
>  note("c4") -- middle C
>  note("c4 #2 v0.5 d0.3") -- middle C with additional properties
>  note({key="c4", volume=0.5}) -- middle C with volume 0.5
>  note("c4'maj v0.7") -- C4 major chord with volume 0.7
>  note("c4", "e4 v0.5", "off") -- custom chord with a c4, e4 and 'off' note
>  ```  



---  
## Aliases  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  



# Note {#Note}  

---  
## Properties
### notes : [`NoteTable`](../API/note.md#NoteTable)[] {#notes}
  

---  
## Functions
### transpose([*self*](../API/builtins/self.md), step : [`integer`](../API/builtins/integer.md) | [`integer`](../API/builtins/integer.md)[]) {#transpose}
`->`[`Note`](../API/note.md#Note)  

> Transpose the notes key with the specified step or steps.
> 
> Values outside of the valid key range (0 - 127) will be clamped.
> 
> #### examples:
> ```lua
> note("c4"):transpose(12)
> note("c'maj"):transpose(5)
> note("c'maj"):transpose({0, 0, -12})
> ```
### amplify([*self*](../API/builtins/self.md), factor : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#amplify}
`->`[`Note`](../API/note.md#Note)  

> Multiply the note's volume attribute with the specified factor or factors.
> 
> Values outside of the valid volume range (0 - 1) will be clamped.
> 
> #### examples:
> ```lua
> note({"c4 0.5", "g4"}):amplify(0.5)
> note("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
> ```
### volume([*self*](../API/builtins/self.md), volume : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#volume}
`->`[`Note`](../API/note.md#Note)  

> Set the note's volume attribute to the specified value or values.
> 
> #### examples:
> ```lua
> note({"c4", "g4"}):volume(0.5)
> note("c'maj"):volume(0.5)
> note("c'maj"):volume({0.1, 0.2, 0.3})
> ```
### instrument([*self*](../API/builtins/self.md), instrument : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#instrument}
`->`[`Note`](../API/note.md#Note)  

> Set the note's instrument attribute to the specified value or values.
### panning([*self*](../API/builtins/self.md), panning : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#panning}
`->`[`Note`](../API/note.md#Note)  

> Set the note's panning attribute to the specified value or values.
### delay([*self*](../API/builtins/self.md), delay : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#delay}
`->`[`Note`](../API/note.md#Note)  

> Set the note's delay attribute to the specified value or values.  



# NoteTable {#NoteTable}  

---  
## Properties
### key : [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) {#key}
> Note Key

### instrument : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#instrument}
> Instrument/Sample/Patch >= 0

### volume : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#volume}
> Volume in range [0.0 - 1.0]

### panning : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#panning}
> Panning factor in range [-1.0 - 1.0] where 0 is center

### delay : [`number`](../API/builtins/number.md)[`?`](../API/builtins/nil.md) {#delay}
> Delay factor in range [0.0 - 1.0]

  



