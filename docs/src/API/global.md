# global  

<!-- toc -->
  

---  
## Functions
### chord(key : [`NoteValue`](#NoteValue), mode : [`ChordName`](#ChordName)) {#chord}
`->`[`Note`](../API/Note.md)  

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
### cycle(input : [`string`](../API/builtins/string.md)) {#cycle}
`->`[`Cycle`](../API/Cycle.md)  

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
### note(...[`NoteValue`](#NoteValue)) {#note}
`->`[`Note`](../API/Note.md)  

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
### rhythm(options : [`RhythmOptions`](../API/RhythmOptions.md)) {#rhythm}
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
>   emit = function(context)
>     local cmin = scale("c5", "pentatonic minor").notes
>     return function(context)
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
### scale(key : [`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md), mode : [`ScaleMode`](#ScaleMode)) {#scale}
`->`[`Scale`](../API/Scale.md)  

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
### sequence(...[`Note`](../API/Note.md) | [`NoteValue`](#NoteValue)) {#sequence}
`->`[`Sequence`](../API/Sequence.md)  

> Create a sequence from an array of note values or note value varargs.
> 
> Using `sequence` instead of a raw `{}` table can be useful to ease transforming the note
> content and to explicitly pass a sequence of e.g. single notes to the emitter.
> 
> #### examples:
> ```lua
> -- sequence of C4, C5 and an empty note
> sequence(48, "c5", {})
> -- sequence of a +5 transposed C4 and G4 major chord
> sequence("c4'maj", "g4'maj"):transpose(5)
>  ```  



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
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/NoteTable.md) | [`nil`](../API/builtins/nil.md)  
  
  
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
  

