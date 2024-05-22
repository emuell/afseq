-- local scale = scale("c5", {0,1,3,5,7,9,11})
local scale = scale("c5", "natural minor")

return rhythm {
  unit = "1/8",
  pattern = pattern.from({ 1, 0, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
  emit = pattern.generate(function(value)
      return { key = scale.notes[value[1]] + (value[2] or 0), volume = 0.7 }
    end,
    { 1 }, { 3 }, { 4 }, { 1 }, { 3 }, { 4 }, { 7, -12 }
  )
}
