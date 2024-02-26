return emitter {
  unit = "seconds",
  resolution = 8,
  offset = 64,
  emit = {
    note("c_4", "---", "---"):with_volume(0.2),
    note("---", "c_5", "---"):with_volume(0.25),
    note("---", "---", "f_5"):with_volume(0.2)
  }
}
