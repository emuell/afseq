---@meta
---
--- Part of the afseq trait:
--- adds a few extra helper functions to the Lua default math lib
---

----------------------------------------------------------------------------------------------------

---Wrap a lua 1 based integer index into the given array/table length.
---
----> `(index - 1) % length + 1`
---@param index integer
---@param length integer
---@return integer
---@nodiscard
function math.imod(index, length)
  return ((index - 1) % length) + 1
end

---* `math.random()`: Returns a float in the range [0,1).
---* `math.random(n)`: Returns a integer in the range [1, n].
---* `math.random(m, n)`: Returns a integer in the range [m, n].
---
---Overridden to use a `Xoshiro256PlusPlus` random number generator to ensure that
-- seeded random operations behave the same on all platforms and architectures.
---
---[View documents](command:extension.lua.doc?["en-us/51/manual.html/pdf-math.random"])
---
---@overload fun():number
---@overload fun(m: integer):integer
---@param m integer
---@param n integer
---@return integer
---@nodiscard
---@diagnostic disable-next-line: redundant-parameter
function math.random(m, n) end

---
---Sets `x` as the "seed" for the pseudo-random generator.
---
---Overridden to seed the internally used  `Xoshiro256PlusPlus` random number generator.
---
---[View documents](command:extension.lua.doc?["en-us/51/manual.html/pdf-math.randomseed"])
---
---@param x integer
function math.randomseed(x) end

---Create a new local random number state with the given optional seed value.
---
---When no seed value is specified, the global `math.randomseed` value is used.
---When no global seed value is available, a new unique random seed is created.
---
---Random states can be useful to create multiple, separate seeded random number
---generators, e.g. in pattern, gate or emit generators, which get reset with the
---generator functions.
---
---### examples:
---
---```lua
---return pattern {
---  event = function(init_context)
---    -- use a unique random sequence every time the pattern gets (re)triggered
---    local rand = math.randomstate(12345)
---    return function(context)
---      if rand(1, 10) > 5 then
---        return "c5"
---      else
---        return "g4"
---      end
---  end
---}
---```
---@param seed? integer
---@return fun(m: integer?, n: integer?): number
function math.randomstate(seed) end
