return emitter {
  unit = "bars",
  resolution = 4,
  offset = 16,
  emit = sequence(
    note("c-4'm"),
    note("g-3'm7"),
    note("c-4'm"),
    note("c-4'm", "g4 0.6"):transpose({0, 0, 3})
  ):amplify({0.5, 0.4, 0.5, 0.4})
}
