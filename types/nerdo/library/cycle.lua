---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Cycle class.
---

----------------------------------------------------------------------------------------------------

---@class Cycle
local Cycle = {}

----------------------------------------------------------------------------------------------------

---Map names in in the cycle to custom note events.
---
---By default, strings in cycles are interpreted as notes, and integer values as MIDI note
---values. Custom identifiers such as "bd" are undefined and will result into a rest, when
---they are not mapped explicitely.
---
---Chords such as "c4'major" are not (yet) supported as values.
---@param map { [string]: NoteValue}
---@return Cycle
---### examples:
---```lua
---cycle("bd [bd sn]"):map({
---  bd = "c4",
---  sn = "e4 #1 v0.2"
---})
---cycle("bd:1 <bd:5, bd:7>"):map({
---  bd = { key = "c4", volume = 0.5 }, -- instrument #1,5,7 will be set as specified
---})
---```
function Cycle:map(map) end

----------------------------------------------------------------------------------------------------

--- Create a note sequence from a Tidal Cycles mini-notation string.
---
--- `cycle` accepts a mini-notation as used by Tidal Cycles, with the following differences:
--- * Stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)
--- * Operators currently only accept numbers on the right side (`a3*2` is valid, `a3*<1 2>` is not)
--- * `:` - Sets the instrument or remappable target instead of selecting samples
--- [Tidal Cycles Reference](https://tidalcycles.org/docs/reference/mini_notation/)
---
---@param input string
---@return Cycle
---### examples:
--- ```lua
-----A chord sequence
---cycle("[c4, e4, g4] [e4, g4, b4] [g4, b4, d5] [b4, d5, f#5]")
-----Arpegio pattern with variations
---cycle("<c4 e4 g4> <e4 g4> <g4 b4 d5> <b4 f5>")
-----Euclidean Rhythms
---cycle("c4(3,8) e4(5,8) g4(7,8)")
-----Polyrhythm
---cycle("{c4 e4 g4 b4}%2, {f4 d4 a4}%4")
-----Custom name mappings
---cycle("bd(3,8)"):map({ bd = "c4 #1" })
--- ```
function cycle(input) end
