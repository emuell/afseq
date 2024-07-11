-- local scale = scale("c5", {0,1,3,5,7,9,11})

return rhythm {
  unit = "1/8",
  pattern = pattern.from({ 1, 0.5, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
  gate = function(context)
    return context.pulse_value == 1.0
  end,
  emit = cycle("<1 3 4 1 3 4 7>"):map(
    mappings.combine(
      mappings.intervals(scale("c5", "natural minor")),
      mappings.transpose({ 0, 0, 0, 0, 0, 0, -12 })
    )
  )
}
