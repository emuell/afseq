---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Emitter class.
---

----------------------------------------------------------------------------------------------------

---@class NoteTable
---@field key string|number Note Key
---@field instrument number? Instrument/Sample/Patch >= 0
---@field volume number? Volume in range [0.0 - 4.0]
---@field panning number? Panning factor in range [-1.0 - 1.0] where 0 is center
---@field delay number? Delay factor in range [0.0 - 1.0]
NoteTable = {}

----------------------------------------------------------------------------------------------------

---@class Note
---@field notes NoteTable[]
Note = {}

---Create a transposed copy of the note or chord.
---@param self Note
---@param step integer|integer[]
---@return Note
---### examples:
---```lua
---note("c4"):transpose(12)
---note("c'maj"):transpose(5)
---note("c'maj"):transpose({0, 0, 5})
---```
function Note.transpose(self, step) end

---Create a copy of the note or chord with amplified volume values.
---@param self Note
---@param factor number|number[] 
---@return Note
---### examples:
---```lua
---note({"c4 0.5", "g4"}):amplify(0.5)
---note("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
---```
function Note.amplify(self, factor) end

---Create a copy of the note or chord with new volume values.
---@param self Note
---@param volume number|number[] 
---@return Note
---### examples:
---```lua
---note({"c4", "g4"}):with_volume(0.5)
---note("c'maj"):with_volume(0.5)
---note("c'maj"):with_volume({0.1, 0.2, 0.3})
---```
function Note.with_volume(self, volume) end

---Create a copy of the note or chord with new panning values.
---@param self Note
---@param panning number|number[] 
---@return Note
function Note.with_delay(self, panning) end

---Create a copy of the note or chord with new panning values.
---@param self Note
---@param panning number|number[] 
---@return Note
function Note.with_delay(self, panning) end

----------------------------------------------------------------------------------------------------

---@alias NoteValue NoteTable|string|number

--- Create a new monophonic or polyphonic note (a chord) from a number value, 
--- a note string, chord string or array of note values.
---
--- In note strings the following prefixes are used to specify optional note 
--- attributes: 
---```md
--- -'#' -> instrument (integer > 0)
--- -'v' -> volume (float in range [0-4])
--- -'p' -> panning (float in range [-1-1])
--- -'d' -> delay (float in range [0-1])
---```
---@param ... NoteValue
---@return Note
--- #### Examples
--- ```lua
--- note(60) -- middle C
--- note("c4") -- middle C
--- note("c4 v0.5 d0.333") -- middle C with volume 0.5 and a delay of 1/3
--- note("c4 #2") -- middle C with instrument/sample/patch 2
--- note({key = "c4", volume = 0.5}) -- middle C with volume 0.5
--- note("c4'maj v0.7 p0.5") -- C4 major chord with volume 0.7, panning 0.5 R
--- note("c4", "e4 v0.5", "off") -- custom chord with a c4, e4 and 'off' note
--- ```
---@overload fun(table: NoteValue[]): Note
---@overload fun(...: NoteValue): Note
function note(...) end