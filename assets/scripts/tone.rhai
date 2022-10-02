let scales = [
  notes_in_scale("c mixolydian"), 
  notes_in_scale("g mixolydian"), 
];
let note_step = 0;
let scale_step = 0;

beat_time()
  .every_nth_sixteenth(1)
  .with_pattern([1, 1, 1, 1])
  .trigger(|| {
    let notes = [0, 2, 3, 5, 2, 3, 5].map(|i| scales[scale_step][i] - 12);
    let note = notes[note_step];
    note_step += 1;
    if note_step >= notes.len() {
      note_step = 0;
      scale_step += 1;
      if scale_step >= scales.len() {
        scale_step = 0;
      }
    }
    let vel = 0.05;
    if note_step % 2 == 0 {
      vel = 0.1;
    }
    [note, vel * 0.5] 
  })
