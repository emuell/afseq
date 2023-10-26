return Emitter {
  unit = "sixteenth",
  pattern = pattern.from({ 1, 0, 0, 0 }, { 0, 0, 1, 0}, {0, 0, 1, 0}, {0, 0, 0, 0} ),
  emit = sequence(60, 60, {60, "C7 0.135"}),
}
