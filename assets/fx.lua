return Emitter {
  unit = "seconds",
  resolution = 8,
  offset = 64,
  emit = sequence(
    chord("C 3", "---", "---"):with_volume(0.2),
    chord("---", "C 4", "---"):with_volume(0.25),
    chord("---", "---", "F 4"):with_volume(0.2)
  )
}