---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq scale function.
---

----------------------------------------------------------------------------------------------------

---@class Scale
---Scale note values as integers, in ascending order of the mode, starting from the scale's key note.
---@field notes integer[]
Scale = {}

---Create a new scale from the given key notes and a mode name.
---@param key string|number e.g. "c4" or 48
---@param mode "chromatic"|"major"|"minor"|"natural major"|"natural minor"|"pentatonic major"|"pentatonic minor"|"pentatonic egyptian"|"blues major"|"blues minor"|"whole tone"|"augmented"|"prometheus"|"tritone"|"harmonic major"|"harmonic minor"|"melodic minor"|"all minor"|"dorian"|"phrygian"|"phrygian dominant"|"lydian"|"lydian augmented"|"mixolydian"|"locrian"|"locrian major"|"super locrian"|"neapolitan major"|"neapolitan minor"|"romanian minor"|"spanish gypsy"|"hungarian gypsy"|"enigmatic"|"overtone"|"diminished half"|"diminished whole"|"spanish eight-tone"|"nine-tone"
---@return Scale
---### example:
---```lua
---scale("c4", "minor").motes -> {"c4", "d4", "d#4", "f4", "g4", "g#4", "a#4"}
---```
function scale(key, mode) end

---Create a new scale instance from the given key and a custom interval table.
---@param key string|number e.g. "c4" or 48
---@param intervals integer[] list of transpose steps relative to the key note
---@return Scale
---### example:
---```lua
---scale("c4", {0,3,5,7}).motes -> {"c4", "d#4", "f4", "g4", "a4"}
---```
function scale(key, intervals) end

---Create a chord from the given degree, build from the scale's notes.
---Skips nth notes from the root as degree, then takes every second note
---from the remaining scale to create a chord.
---By default a triad is created.
---@param degree integer|"i"|"ii"|"iii"|"iv"|"v"|"vi"|"vii"
---@param note_count integer?
---@return integer[] notes
---### example:
---```lua
---local cmin = scale("c4", "minor")
---cmin:chord("i", 4) --> {48, 51, 55, 58}
---note(cmin:chord(5)):transposed({12, 0, 0}) --> Gm 1st inversion
---```
function Scale:chord(degree, note_count) end

---Fit given note value(s) by moving it to the nearest note in the scale.
---@param ... NoteValue
---@return integer[]
---### example:
---```lua
---local cmin = scale("c4", "minor")
---cmin:fit("c4", "d4", "f4") -> 48, 50, 53 (cmin)
---```
function Scale:fit(...) end
