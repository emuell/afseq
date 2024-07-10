---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq scale function.
---

----------------------------------------------------------------------------------------------------

---Roman number or plain number as degree in range [1 - 7]
---@alias DegreeValue integer|"i"|"ii"|"iii"|"iv"|"v"|"vi"|"vii"

---@class Scale
---Scale note values as integers, in ascending order of the mode, starting from the scale's key note.
---@field notes integer[]
local Scale = {}

---Create a chord from the given degree, built from the scale's intervals.
---Skips nth notes from the root as degree, then takes every second note
---from the remaining scale to create a chord. By default a triad is created.
---
---### examples:
---```lua
---local cmin = scale("c4", "minor")
---cmin:chord("i", 4) --> {48, 51, 55, 58}
---note(cmin:chord(5)):transpose({12, 0, 0}) --> Gm 1st inversion
---```
---@param degree DegreeValue Degree value in range [1..7]
---@param note_count integer? Number of notes in chord. By default 3.
---@return integer[] notes
---@nodiscard
function Scale:chord(degree, note_count) end

---Get a single or multiple notes by its degree from the scale, using the given roman
---number string or a plain number as interval index.
---Allows picking intervals from the scale to e.g. create chords with roman number
---notation.
---
---### examples:
---```lua
---local cmmaj = scale("c4", "major")
---cmin:degree(1) --> 48 ("c4")
---cmin:degree(5) --> 55
---cmin:degree("i", "iii", "v") --> 48, 50, 55
---```
---@param ... DegreeValue Degree value(s) in range [1..7]
---@return integer ...
---@nodiscard
function Scale:degree(...) end

---Fit given note value(s) into scale by moving them to the nearest note in the scale.
---
---### examples:
---```lua
---local cmin = scale("c4", "minor")
---cmin:fit("c4", "d4", "f4") -> 48, 50, 53 (cmaj -> cmin)
---```
---@param ... NoteValue
---@return integer[]
---@nodiscard
function Scale:fit(...) end

----------------------------------------------------------------------------------------------------

---Available scales.
---@alias ScaleMode "chromatic"|"major"|"minor"|"natural major"|"natural minor"|"pentatonic major"|"pentatonic minor"|"pentatonic egyptian"|"blues major"|"blues minor"|"whole tone"|"augmented"|"prometheus"|"tritone"|"harmonic major"|"harmonic minor"|"melodic minor"|"all minor"|"dorian"|"phrygian"|"phrygian dominant"|"lydian"|"lydian augmented"|"mixolydian"|"locrian"|"locrian major"|"super locrian"|"neapolitan major"|"neapolitan minor"|"romanian minor"|"spanish gypsy"|"hungarian gypsy"|"enigmatic"|"overtone"|"diminished half"|"diminished whole"|"spanish eight-tone"|"nine-tone"|string

---Create a new scale from the given key notes and a mode name.
---
---Scale names can also be shortened by using the following synonyms:
---- "8-tone" -> "eight-tone"
---- "9-tone" -> "nine-tone"
---- "aug" -> "augmented"
---- "dim" -> "diminished"
---- "dom" -> "Dominant"
---- "egypt"  -> "egyptian"
---- "harm"  -> "harmonic"
---- "hungary" -> "hungarian"
---- "roman" -> "romanian"
---- "min" -> "minor"
---- "maj" -> "major"
---- "nat" -> "natural"
---- "penta" -> "pentatonic"
---- "span" -> "spanish",
---
---### examples:
---```lua
---scale("c4", "minor").notes -> {"c4", "d4", "d#4", "f4", "g4", "g#4", "a#4"}
---```
---@param key string|number e.g. "c4" or 48
---@param mode ScaleMode
---@return Scale
---@nodiscard
function scale(key, mode) end

---Create a new scale instance from the given key and a custom interval table.
---
---### examples:
---```lua
---scale("c4", {0,3,5,7}).notes -> {"c4", "d#4", "f4", "g4", "a4"}
---```
---@param key string|number e.g. "c4" or 48
---@param intervals integer[] list of transpose steps relative to the key note
---@return Scale
---@nodiscard
function scale(key, intervals) end
