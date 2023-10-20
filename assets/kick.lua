return Emitter {
  unit = "beats",
  resolution = 1 / 4,
  pattern = { 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0 },
  -- emit = "C4",
  emit = sequence(60, 60, chord(60, {key = "C7", volume = 0.15})),
  -- emit = sequence("C4", "C5", "-"),
  -- emit = sequence(chord("C4", "C5", "-"), chord("C4", "C5", "-"), chord("C4", "C5", "-")),
  -- emit = chord({ key = "C4" }, { key = "C5", volume = 0.5 }),
  -- emit = sequence { { key = "C4" }, { key = "C5", volume = 0.5 } },
  -- emit = lerp(param("fx1", "resonance"), 0.0, 1.0, 10),
}
