return Emitter {
  unit = "bars",
  resolution = 4,
  offset = 16,
  emit = sequence(
    note("C_4", "D#4", "G_4"):with_volume(0.3),
    note("C_4", "D#4", "F_4"):with_volume(0.4),
    note("C_4", "D#4", "G_4"):with_volume(0.3),
    note("C_4", "D#4", "A#4"):with_volume(0.4)
  )
}
