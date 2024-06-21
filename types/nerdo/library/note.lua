---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Note class.
---

----------------------------------------------------------------------------------------------------

---@class NoteTable
---@field key string|number Note Key
---@field instrument number? Instrument/Sample/Patch >= 0
---@field volume number? Volume in range [0.0 - 1.0]
---@field panning number? Panning factor in range [-1.0 - 1.0] where 0 is center
---@field delay number? Delay factor in range [0.0 - 1.0]
local NoteTable = {}

----------------------------------------------------------------------------------------------------

---@class Note
---@field notes NoteTable[]
local Note = {}

---Create a transposed copy of the note or chord.
---
---### examples:
---```lua
---note("c4"):transposed(12)
---note("c'maj"):transposed(5)
---note("c'maj"):transposed({0, 0, 5})
---```
---@param step integer|integer[]
---@return Note
---@nodiscard
function Note:transposed(step) end

---Create a copy of the note or chord with amplified volume values.
---
---### examples:
---```lua
---note({"c4 0.5", "g4"}):amplified(0.5)
---note("c'maj 0.5"):amplified({2.0, 1.0, 0.3})
---```
---@param factor number|number[] amplify factor > 0
---@return Note
---@nodiscard
function Note:amplified(factor) end

---Create a copy of the note or chord with new volume values.
---
---### examples:
---```lua
---note({"c4", "g4"}):with_volume(0.5)
---note("c'maj"):with_volume(0.5)
---note("c'maj"):with_volume({0.1, 0.2, 0.3})
---```
---@param volume number|number[] new volume in range [0 - 1]
---@return Note
---@nodiscard
function Note:with_volume(volume) end

---Create a copy of the note or chord with new instrument values.
---@param instrument number|number[] 
---@return Note
---@nodiscard
function Note:with_instrument(instrument) end

---Create a copy of the note or chord with new panning values.
---@param panning number|number[] 
---@return Note
---@nodiscard
function Note:with_panning(panning) end

---Create a copy of the note or chord with new delay values.
---@param delay number|number[] 
---@return Note
---@nodiscard
function Note:with_delay(delay) end

----------------------------------------------------------------------------------------------------

---@alias NoteValue NoteTable|string|number|nil

--- Create a new monophonic or polyphonic note (a chord) from a number value, 
--- a note string, chord string or array of note values.
---
--- In note strings the following prefixes are used to specify optional note 
--- attributes: 
---```md
--- -'#' -> instrument (integer > 0)
--- -'v' -> volume (float in range [0-1])
--- -'p' -> panning (float in range [-1-1])
--- -'d' -> delay (float in range [0-1])
---```
---
---### examples:
--- ```lua
--- note(60) -- middle C
--- note("c4") -- middle C
--- note("c4 #2 v0.5 d0.3") -- middle C with additional properties
--- note({key="c4", volume=0.5}) -- middle C with volume 0.5
--- note("c4'maj v0.7") -- C4 major chord with volume 0.7
--- note("c4", "e4 v0.5", "off") -- custom chord with a c4, e4 and 'off' note
--- ```
---@param ... NoteValue
---@return Note
---@nodiscard
---@overload fun(table: NoteValue[]): Note
---@overload fun(...: NoteValue): Note
function note(...) end
