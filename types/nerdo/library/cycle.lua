---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Cycle class.
---

----------------------------------------------------------------------------------------------------

---@class Cycle
local Cycle = {}

----------------------------------------------------------------------------------------------------

--- Create a note sequence from a Tidal Cycles mini-notation string.
---
--- `cycle` accepts a mini-notation as used by Tidal Cycles, with the following differences:
--- * Stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)
--- * Operators currently only accept numbers on the right side (`a3*2` is valid, `a3*<1 2>` is not)
--- * Polymeters only work by specifying the subdivision on the right (`{a b c d}%3`)
--- * Random event muting always requires a probability (ie `a3?0.5` instead of `a3?`)
--- * `:` - Sets the instrument or remappable target instead of selecting samples
--- * `/` - Slow pattern operator is not implemented yet
--- * `@` - Elongate operator is missing (use explicit `_` steps instead)
--- * `.` - Shorthand for groups is missing (use `[a b] [c d]` instead of `a b . c d`)
---
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
--- Polyrhythm
---cycle("{c4 e4 g4 b4}%2, {f4 d4 a4}%4")
--- ```
function cycle(input) end
