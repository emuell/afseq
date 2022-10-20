let note_step = 0;

beat_time()
  .every_sixteenth()
  .with_offset(16 * 8)
  .with_pattern(euclidian(11, 16).rotate(6))
  .trigger(|| {
    note_step += 1;
    let vel = 1.0 - (note_step % 4) / 4.0;
    ["C_5", vel]
  })