let note_step = 0;
beat_time()
  .every_nth_sixteenth(1)
  .with_pattern(12, 16)
  .trigger(|| {
    note_step += 1;
    let vel = 1.0 - (note_step % 2) * 0.8;
    ["G_4", vel]
  });