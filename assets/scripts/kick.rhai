
beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([1,0,0,0].repeat(15) + [1,0,1,0])
  .trigger(note([["C_3", 0.8], ["C_5", 0.2]]));