
<!-- toc -->

# Global {#Global}  

---  
## Functions
### chord(key : [`NoteValue`](#NoteValue), mode : [`ChordName`](#ChordName)) {#chord}
`->`[`Note`](../API/note.md#Note)  

> Create a new chord from the given key notes and a chord name or an array of custom intervals.
> 
> NB: Chords also can also be defined via strings in function `note` and via the a scale's 
> `chord` function. See examples below. 
> 
> Chord names also can be shortened by using the following synonyms:
> - "min" | "m" | "-" -> "minor"
> - "maj" | "M" | "^" -> "major"
> - "minMaj" | "mM" | "-^" -> "minMajor"
> - "+" | "aug" -> "augmented"
> - "o" | "dim" -> "diminished"
> - "5 -> "five"
> - "6 -> "six"
> - "69" -> "sixNine"
> - "9 -> "nine"
> - "11" -> "eleven"
> 
> #### examples:
> ```lua
> chord("c4", "minor") --> {"c4", "d#4", "f4"}
> chord({key = 48, volume = 0.5}, "minor") --> {"c4 v0.5", "d#4 v0.5", "f4 v0.5"}
> --same as:
> chord("c4", {0, 4, 7})
> chord("c4 v0.5", {0, 4, 7})
> --or:
> note("c4'major")
> note("c4'major v0.5")
> --or:
> note(scale("c4", "major"):chord("i", 3))
> note(scale("c4", "major"):chord("i", 3)):volume(0.5)
> ```
> 
> ---
> 
> ```lua
> -- Available chords.
> mode:
>     | "major"
>     | "augmented"
>     | "six"
>     | "sixNine"
>     | "major7"
>     | "major9"
>     | "add9"
>     | "major11"
>     | "add11"
>     | "major13"
>     | "add13"
>     | "dom7"
>     | "dom9"
>     | "dom11"
>     | "dom13"
>     | "7b5"
>     | "7#5"
>     | "7b9"
>     | "9"
>     | "nine"
>     | "eleven"
>     | "thirteen"
>     | "minor"
>     | "diminished"
>     | "dim"
>     | "minor#5"
>     | "minor6"
>     | "minor69"
>     | "minor7b5"
>     | "minor7"
>     | "minor7#5"
>     | "minor7b9"
>     | "minor7#9"
>     | "diminished7"
>     | "minor9"
>     | "minor11"
>     | "minor13"
>     | "minorMajor7"
>     | "five"
>     | "sus2"
>     | "sus4"
>     | "7sus2"
>     | "7sus4"
>     | "9sus2"
>     | "9sus4"
> ```  



---  
## Aliases  
### ChordName {#ChordName}
[`string`](../API/builtins/string.md) | `"7#5"` | `"7b5"` | `"7b9"` | `"7sus2"` | `"7sus4"` | `"9"` | `"9sus2"` | `"9sus4"` | `"add11"` | `"add13"` | `"add9"` | `"augmented"` | `"dim"` | `"diminished"` | `"diminished7"` | `"dom11"` | `"dom13"` | `"dom7"` | `"dom9"` | `"eleven"` | `"five"` | `"major"` | `"major11"` | `"major13"` | `"major7"` | `"major9"` | `"minor"` | `"minor#5"` | `"minor11"` | `"minor13"` | `"minor6"` | `"minor69"` | `"minor7"` | `"minor7#5"` | `"minor7#9"` | `"minor7b5"` | `"minor7b9"` | `"minor9"` | `"minorMajor7"` | `"nine"` | `"six"` | `"sixNine"` | `"sus2"` | `"sus4"` | `"thirteen"`  
> ```lua
> -- Available chords.
> ChordName:
>     | "major"
>     | "augmented"
>     | "six"
>     | "sixNine"
>     | "major7"
>     | "major9"
>     | "add9"
>     | "major11"
>     | "add11"
>     | "major13"
>     | "add13"
>     | "dom7"
>     | "dom9"
>     | "dom11"
>     | "dom13"
>     | "7b5"
>     | "7#5"
>     | "7b9"
>     | "9"
>     | "nine"
>     | "eleven"
>     | "thirteen"
>     | "minor"
>     | "diminished"
>     | "dim"
>     | "minor#5"
>     | "minor6"
>     | "minor69"
>     | "minor7b5"
>     | "minor7"
>     | "minor7#5"
>     | "minor7b9"
>     | "minor7#9"
>     | "diminished7"
>     | "minor9"
>     | "minor11"
>     | "minor13"
>     | "minorMajor7"
>     | "five"
>     | "sus2"
>     | "sus4"
>     | "7sus2"
>     | "7sus4"
>     | "9sus2"
>     | "9sus4"
> ```  
  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  


