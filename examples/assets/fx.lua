return rhythm {
  unit = "seconds",
  resolution = 8,
  offset = 8,
  emit = {
    note("c_4", "---", "---"):volume(0.2),
    note("---", "c_5", "---"):volume(0.25),
    note("---", "---", "f_5"):volume(0.2)
  }
}
