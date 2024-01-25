---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq scale function.
---

----------------------------------------------------------------------------------------------------

---@class Scale
---Scale note values in ascending order of the mode, starting from the scale's key note.  
---@field notes integer[]
Scale = {}

---Create a new scale from the given key notes and a mode name. 
---@param key string|number e.g. "c4" or 48
---@param mode "chromatic"|"natural major"|"natural minor"|"pentatonic major"|"pentatonic minor"|"pentatonic egyptian"|"blues major"|"blues minor"|"whole tone"|"augmented"|"prometheus"|"tritone"|"harmonic major"|"harmonic minor"|"melodic minor"|"all minor"|"dorian"|"phrygian"|"phrygian dominant"|"lydian"|"lydian augmented"|"mixolydian"|"locrian"|"locrian major"|"super locrian"|"neapolitan major"|"neapolitan minor"|"romanian minor"|"spanish gypsy"|"hungarian gypsy"|"enigmatic"|"overtone"|"diminished half"|"diminished whole"|"spanish eight-tone"|"nine-tone"
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
