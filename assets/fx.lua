return Emitter {
  unit = "seconds",
  resolution = 8,
  offset = 64,
  emit = sequence(
    note("C 3", "---", "---"):with_volume(0.2),
    note("---", "C 4", "---"):with_volume(0.25),
    note("---", "---", "F 4"):with_volume(0.2)
  )
}