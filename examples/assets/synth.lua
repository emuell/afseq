local scale = scale("c4", "minor")

return emitter {
  unit = "bars",
  resolution = 4,
  offset = 4,
  pattern = {1, 1, 1, 1},
  repeats = 3,
  emit = sequence(
    note(scale:chord("i", 3)),
    note(scale:chord("v", 4)):transpose(-12),
    note(scale:chord("i", 3)),
    note(scale:chord("v", 4)):transpose({-12, -12, 0, -12})
  ):amplify({ 0.6, 0.5, 0.6, 0.5 })
}
