beat_time()
  .every_nth_bar(4.0)
  .with_offset(16.0)
  .trigger(sequence([
      ["C 3 0.3", "D#3 0.3", "G 3 0.3"],
      ["C 3 0.3", "D#3 0.3", "F 3 0.3"],
      ["C 3 0.3", "D#3 0.3", "G 3 0.3"],
      ["C 3 0.3", "D#3 0.3", "A#3 0.3"],
  ]));