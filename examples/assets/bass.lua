-- local scale = scale("c5", {0,1,3,5,7,9,11})
local scale = scale("c5", "natural minor")

return rhythm {
  unit = "1/8",
  pattern = pattern.from({ 1, 0.5, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
  gate = function(context)
    return context.pulse_value == 1.0
  end,
  emit = pattern.from(1, 3, 4, 1, 3, 4, -7):map(function(index, value)
    if value < 0 then
      return { key = scale.notes[-value] - 12, volume = 0.7 }
    else
      return { key = scale.notes[value], volume = 0.7 }
    end
  end
  )
}
