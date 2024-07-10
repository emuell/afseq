---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Sequence class.
---

----------------------------------------------------------------------------------------------------

---@class Sequence
---@field notes NoteTable[][]
local Sequence = {}

---Create a copy of all notes in the sequence with transposed note values.
---
---### examples:
---```lua
---sequence("c4", "d#5"):transpose(12)
---sequence(note("c'maj"), note("c'maj")):transpose({0, 5})
---```
---@param step integer|integer[]
---@return Sequence
---@nodiscard
function Sequence:transpose(step) end

---Create a copy of all notes in the sequence with amplified volume values.
---
---### examples:
---```lua
---sequence({"c4 0.5", "g4"}):amplify(0.5)
---sequence("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
---```
---@param factor number|number[]
---@return Sequence
---@nodiscard
function Sequence:amplify(factor) end

---Create a copy of all notes in the sequence with new instrument values.
---@param instrument number|number[]
---@return Note
---@nodiscard
function Sequence:instrument(instrument) end

---Create a copy of all notes in the sequence with new volume values.
---
---### examples:
---```lua
---sequence({"c4", "g4"}):volume(0.5)
---sequence("c'maj"):volume(0.5)
---sequence("c'maj"):volume({0.1, 0.2, 0.3})
---```
---@param volume number|number[]
---@return Sequence
---@nodiscard
function Sequence:volume(volume) end

---Create a copy of all notes in the sequence with new panning values.
---@param panning number|number[]
---@return Note
---@nodiscard
function Sequence:panning(panning) end

---Create a copy of all notes in the sequence with new delay values.
---@param delay number|number[]
---@return Sequence
---@nodiscard
function Sequence:delay(delay) end

----------------------------------------------------------------------------------------------------

---Create a sequence from an array of note values or note value varargs.
---
---Using `sequence` instead of a raw `{}` table can be useful to ease transforming the note
---content and to explicitly pass a sequence of e.g. single notes to the emitter.
---
---### examples:
---```lua
---sequence(48, "c5", {}) -- sequence of C4, C5 and an empty note
---sequence("c4'maj", "g4'maj"):transpose(5) -- sequence of a +5 transposed C4 and G4 major chord
--- ```
---@param ... NoteValue|Note
---@return Sequence
---@nodiscard
---@overload fun(table: (NoteValue|Note)[]): Sequence
---@overload fun(...: (NoteValue|Note)): Sequence
function sequence(...) end
