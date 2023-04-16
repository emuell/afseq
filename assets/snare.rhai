beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([0,0,0,0, 1,0,0,0].repeat(7) + [0,0,0,1, 0,0,1,0])
  .trigger(note("C_4 1.4"));