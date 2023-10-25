let note_step = 0;

beat_time()
  .every_sixteenth()
  .with_offset(16 * 8)
  .with_pattern(euclidean(14, 16).rotate(6))
  .trigger(|| {
    note_step += 1;
    let volume = 1.0 - (note_step % 4) / 4.0;
    let note = "C 5";
    if (note_step % 3 == 0) { 
      note = "C 4"; 
      volume *= 0.6;
    }
    `${note} ${volume}`
  })