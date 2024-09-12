# Scale  

<!-- toc -->
  

---  
## Properties
### notes : [`integer`](../API/builtins/integer.md)[] {#notes}
> Scale note values as integers, in ascending order of the mode, starting from the scale's key note.

  

---  
## Functions
### chord([*self*](../API/builtins/self.md), degree : [`DegreeValue`](#DegreeValue), note_count : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md)) {#chord}
`->`notes : [`integer`](../API/builtins/integer.md)[]  

> Create a chord from the given degree, built from the scale's intervals.
> Skips nth notes from the root as degree, then takes every second note
> from the remaining scale to create a chord. By default a triad is created.
> 
> #### examples:
> ```lua
> local cmin = scale("c4", "minor")
> cmin:chord("i", 4) --> {48, 51, 55, 58}
> note(cmin:chord(5)):transpose({12, 0, 0}) --> Gm 1st inversion
> ```
> 
> ```lua
> -- Roman number or plain number as degree in range [1 - 7]
> degree:
>     | "i"
>     | "ii"
>     | "iii"
>     | "iv"
>     | "v"
>     | "vi"
>     | "vii"
> ```
### degree([*self*](../API/builtins/self.md), ...[`DegreeValue`](#DegreeValue)) {#degree}
`->`... : [`integer`](../API/builtins/integer.md)  

> Get a single or multiple notes by its degree from the scale, using the given roman
> number string or a plain number as interval index.
> Allows picking intervals from the scale to e.g. create chords with roman number
> notation.
> 
> #### examples:
> ```lua
> local cmmaj = scale("c4", "major")
> cmin:degree(1) --> 48 ("c4")
> cmin:degree(5) --> 55
> cmin:degree("i", "iii", "v") --> 48, 50, 55
> ```
> 
> ```lua
> -- Roman number or plain number as degree in range [1 - 7]
> ...(param):
>     | "i"
>     | "ii"
>     | "iii"
>     | "iv"
>     | "v"
>     | "vi"
>     | "vii"
> ```
### fit([*self*](../API/builtins/self.md), ...[`NoteValue`](#NoteValue)) {#fit}
`->`[`integer`](../API/builtins/integer.md)[]  

> Fit given note value(s) into scale by moving them to the nearest note in the scale.
> 
> #### examples:
> ```lua
> local cmin = scale("c4", "minor")
> cmin:fit("c4", "d4", "f4") -> 48, 50, 53 (cmaj -> cmin)
> ```  

