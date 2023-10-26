
beat_time()
  .every_sixteenth()
  .with_pattern([1, 0, 0, 0] + [0, 0, 1, 0] + [0, 0, 1, 0] + [0, 0, 0, 0])
  .trigger(note(["C_4 0.8", "C_5 0.2"]));