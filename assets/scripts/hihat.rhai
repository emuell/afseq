beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([1,0,1,0].repeat(7) + [1,0,1,1])
  .trigger(note_seq(
    [["C_5", 0.25], ["G_4", 1.0]].repeat(7) + [["C_5", 0.25], ["G_4", 1.0], ["C_5", 1.0]],
  ));
