local cmin = scale("c4", "minor")

return rhythm {
  unit = "bars",
  resolution = 4,
  offset = 4,
  pattern = {1, 1, 1, 1},
  repeats = 3,
  emit = sequence(
    note(cmin:chord("i", 3)),
    note(cmin:chord("i", 3)):transposed({0, 0, -2}),
    note(cmin:chord("i", 3)),
    note(cmin:chord("i", 4)):transposed({0, 0, 3, -12})
  ):amplified({ 0.6, 0.5, 0.6, 0.5 })
}
