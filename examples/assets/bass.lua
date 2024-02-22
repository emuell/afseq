local fun = require "fun"
local pattern = require "pattern"

-- local scale = scale("c5", {0,1,3,5,7,9,11}).notes
local scale = scale("c5", "natural minor").notes

local bassline = fun.iter({ { 1 }, { 3 }, { 4 }, { 1 }, { 3 }, { 4 }, { 7, -12 } })
    :map(function(x) return scale[x[1]] + (x[2] or 0) end):to_table()

return emitter {
    unit = "1/8",
    pattern = pattern.from({ 1, 0, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
    emit = sequence(bassline):with_volume(0.7)
}
