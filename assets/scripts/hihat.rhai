let note_step = 0;
beat_time()
  .every_nth_sixteenth(1)
  .with_pattern(11, 16)
  .trigger(|| {
    note_step += 1;
    let vel = 1.0 - (note_step % 4) / 4.0;
    ["C_5", vel]
  });