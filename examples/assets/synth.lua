local cmin = scale("c4", "minor")

return pattern {
  unit = "bars",
  resolution = 4,
  offset = 4,
  pulse = {1, 1, 1, 1},
  repeats = 3,
  event = sequence(
    note(cmin:chord("i", 3)),
    note(cmin:chord("i", 3)):transpose({0, 0, -2}),
    note(cmin:chord("i", 3)),
    note(cmin:chord("i", 4)):transpose({0, 0, 3, -12})
  ):amplify({ 0.6, 0.5, 0.6, 0.5 })
}
