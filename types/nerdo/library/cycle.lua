---@meta
---Do not try to execute this file. It's just a type definition file.
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Cycle class.
---

----------------------------------------------------------------------------------------------------

---@class Cycle
Cycle = {}

----------------------------------------------------------------------------------------------------

--- Create a note sequence from a tidal cycle mini notation string.
---
--- [Tidal Cycles Reference](https://tidalcycles.org/docs/reference/mini_notation/)
--- Everything works the same except for the following differences:
--- * Operators currently only accept number on the right side (`a3*2` is valid, `a3*<1 2>` is not)
--- * Polymeters always require specifying the subdivision on the right (`{a b c d}%3`)
--- * Random event muting always requires a probability (ie `a3?0.5` instead of `a3?`)
--- * `:` - Sets the instrument or remappable target instead of selecting samples
--- * `/` - Slow pattern operator is not implemented yet
--- * `@` - Elongate operator is missing (use explicit `_` steps instead)
--- * `.` - Shorthand for groups is missing (use `[a b] [c d]` instead)
--- * The outermost level is treated as a group, stacks and random choices are valid without brackets (`a | b` is parsed as `[a | b]`)
---
---@param input string
---@return Cycle
---### examples:
--- ```lua
--- TODO
--- ```
function cycle(input) end
