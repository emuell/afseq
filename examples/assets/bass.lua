local fun = require "fun"
local pattern = require "pattern"

-- local scale = scale("c5", {0,1,3,5,7,9,11})
local scale = scale("c5", "natural minor")

--[[
math.randomseed(125231291212)

local rand_note_gen = function(param, state)
  local min, max = param[1], param[2]
  return scale:fit(math.random(min, max)) + 12, state
end

local bassline = fun.wrap(rand_note_gen, {40, 60}):take(8):cycle()
]]

local bassline = fun.iter({ { 1 }, { 3 }, { 4 }, { 1 }, { 3 }, { 4 }, { 7, -12 } })
  :map(function(x)
      return { key = scale.notes[x[1]] + (x[2] or 0), volume = 0.7 }
    end)
  :cycle()

return rhythm {
  unit = "1/8",
  pattern = pattern.from({ 1, 0, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
  emit = bassline
}
