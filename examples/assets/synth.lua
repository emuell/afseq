local cmin = scale("c4", "minor")

return rhythm {
  unit = "bars",
  resolution = 4,
  offset = 4,
  pattern = {1, 1, 1, 1},
  repeats = 3,
  emit = sequence(
    note(cmin:chord("i", 3)),
    note(cmin:chord("v", 4)):transposed(-12),
    note(cmin:chord("i", 3)),
    note(cmin:chord("v", 4)):transposed({-12, -12, 0, -12})
  ):amplified({ 0.6, 0.5, 0.6, 0.5 })
}
