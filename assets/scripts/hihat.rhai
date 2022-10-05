beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([1,0,1,0].repeat(15) + [1,1,1,1])
  .trigger(note_seq(
    [["C_5", 0.25], ["G_4", 1.0]].repeat(15) + [["C_5", 0.25],  ["G_4", 0.5], ["G_4", 1.0], ["G_4", 1.0]],
  ));