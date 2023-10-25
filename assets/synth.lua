return Emitter {
  unit = "bars",
  resolution = 4,
  offset = 16,
  emit = sequence(
    chord("C_3", "D#3", "G_3"):with_volume(0.3),
    chord("C_3", "D#3", "F_3"):with_volume(0.4),
    chord("C_3", "D#3", "G_3"):with_volume(0.3),
    chord("C_3", "D#3", "A#3"):with_volume(0.4)
  )
}
