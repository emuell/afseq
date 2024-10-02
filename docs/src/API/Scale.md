
<!-- toc -->

# Global {#Global}  

---  
## Functions
### scale(key : [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md), mode : [`ScaleMode`](#ScaleMode)) {#scale}
`->`[`Scale`](../API/scale.md#Scale)  

> Create a new scale from the given key notes and a mode name.
> 
> Scale names can also be shortened by using the following synonyms:
> - "8-tone" -> "eight-tone"
> - "9-tone" -> "nine-tone"
> - "aug" -> "augmented"
> - "dim" -> "diminished"
> - "dom" -> "Dominant"
> - "egypt"  -> "egyptian"
> - "harm"  -> "harmonic"
> - "hungary" -> "hungarian"
> - "roman" -> "romanian"
> - "min" -> "minor"
> - "maj" -> "major"
> - "nat" -> "natural"
> - "penta" -> "pentatonic"
> - "span" -> "spanish",
> 
> #### examples:
> ```lua
> scale("c4", "minor").notes -> {"c4", "d4", "d#4", "f4", "g4", "g#4", "a#4"}
> ```
> 
> ```lua
> -- Available scales.
> mode:
>     | "chromatic"
>     | "major"
>     | "minor"
>     | "natural major"
>     | "natural minor"
>     | "pentatonic major"
>     | "pentatonic minor"
>     | "pentatonic egyptian"
>     | "blues major"
>     | "blues minor"
>     | "whole tone"
>     | "augmented"
>     | "prometheus"
>     | "tritone"
>     | "harmonic major"
>     | "harmonic minor"
>     | "melodic minor"
>     | "all minor"
>     | "dorian"
>     | "phrygian"
>     | "phrygian dominant"
>     | "lydian"
>     | "lydian augmented"
>     | "mixolydian"
>     | "locrian"
>     | "locrian major"
>     | "super locrian"
>     | "neapolitan major"
>     | "neapolitan minor"
>     | "romanian minor"
>     | "spanish gypsy"
>     | "hungarian gypsy"
>     | "enigmatic"
>     | "overtone"
>     | "diminished half"
>     | "diminished whole"
>     | "spanish eight-tone"
>     | "nine-tone"
> ```  



---  
## Aliases  
### DegreeValue {#DegreeValue}
[`integer`](../API/builtins/integer.md) | `"i"` | `"ii"` | `"iii"` | `"iv"` | `"v"` | `"vi"` | `"vii"`  
> ```lua
> -- Roman number or plain number as degree in range [1 - 7]
> DegreeValue:
>     | "i"
>     | "ii"
>     | "iii"
>     | "iv"
>     | "v"
>     | "vi"
>     | "vii"
> ```  
  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  
### ScaleMode {#ScaleMode}
[`string`](../API/builtins/string.md) | `"all minor"` | `"augmented"` | `"blues major"` | `"blues minor"` | `"chromatic"` | `"diminished half"` | `"diminished whole"` | `"dorian"` | `"enigmatic"` | `"harmonic major"` | `"harmonic minor"` | `"hungarian gypsy"` | `"locrian major"` | `"locrian"` | `"lydian augmented"` | `"lydian"` | `"major"` | `"melodic minor"` | `"minor"` | `"mixolydian"` | `"natural major"` | `"natural minor"` | `"neapolitan major"` | `"neapolitan minor"` | `"nine-tone"` | `"overtone"` | `"pentatonic egyptian"` | `"pentatonic major"` | `"pentatonic minor"` | `"phrygian dominant"` | `"phrygian"` | `"prometheus"` | `"romanian minor"` | `"spanish eight-tone"` | `"spanish gypsy"` | `"super locrian"` | `"tritone"` | `"whole tone"`  
> ```lua
> -- Available scales.
> ScaleMode:
>     | "chromatic"
>     | "major"
>     | "minor"
>     | "natural major"
>     | "natural minor"
>     | "pentatonic major"
>     | "pentatonic minor"
>     | "pentatonic egyptian"
>     | "blues major"
>     | "blues minor"
>     | "whole tone"
>     | "augmented"
>     | "prometheus"
>     | "tritone"
>     | "harmonic major"
>     | "harmonic minor"
>     | "melodic minor"
>     | "all minor"
>     | "dorian"
>     | "phrygian"
>     | "phrygian dominant"
>     | "lydian"
>     | "lydian augmented"
>     | "mixolydian"
>     | "locrian"
>     | "locrian major"
>     | "super locrian"
>     | "neapolitan major"
>     | "neapolitan minor"
>     | "romanian minor"
>     | "spanish gypsy"
>     | "hungarian gypsy"
>     | "enigmatic"
>     | "overtone"
>     | "diminished half"
>     | "diminished whole"
>     | "spanish eight-tone"
>     | "nine-tone"
> ```  
  



# Scale {#Scale}  

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
> local cmin = scale("c4", "minor")
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
### notes_iter([*self*](../API/builtins/self.md), count : [`integer`](../API/builtins/integer.md)[`?`](../API/builtins/nil.md)) {#notes_iter}
`->`() `->` [`integer`](../API/builtins/integer.md) | [`nil`](../API/builtins/nil.md)  

> Create an iterator function that returns up to `count` notes from the scale.
> If the count exceeds the number of notes in the scale, then notes from the next
> octave are taken. 
> 
> The iterator function returns nil when the maximum number of MIDI notes has been
> reached, or when the given optional `count` parameter has been exceeded.
> 
> #### examples:
> ```lua
> --collect 16 notes of a c major scale
> local cmaj = scale("c4", "major")
> local notes = {}
> for note in cmin:notes_iter(16) do
>  table.insert(notes, note)
> end
> -- same using the `pattern` library
> local notes = pattern.new(16):init(cmaj.notes_iter())
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



---  
## Aliases  
### DegreeValue {#DegreeValue}
[`integer`](../API/builtins/integer.md) | `"i"` | `"ii"` | `"iii"` | `"iv"` | `"v"` | `"vi"` | `"vii"`  
> ```lua
> -- Roman number or plain number as degree in range [1 - 7]
> DegreeValue:
>     | "i"
>     | "ii"
>     | "iii"
>     | "iv"
>     | "v"
>     | "vi"
>     | "vii"
> ```  
  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  



