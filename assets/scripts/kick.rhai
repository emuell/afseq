
beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([1,0,0,0].repeat(15) + [1,0,1,0])
  .trigger(note([["C_4", 1.0], ["C_6", 0.1]].map(|v| {v[0] = "C_2"; v})));
