let scales = [
  notes_in_scale("c mixolydian"), 
  notes_in_scale("g mixolydian"), 
];
let note_step = 0;
let scale_step = 0;
let volume_step = 63;
let volume_direction = 1.0;
let transpose_step = 0;

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
    let vel = 0.3 * volume_step / 128.0 + 0.1;
    if note_step % 2 == 0 {
      vel = 0.3 * (1.0 - volume_step / 64) + 0.1;
    }
    volume_step += volume_direction;
    if volume_step >= 64 || volume_step == 0 {
      volume_direction = -volume_direction;
    }
    let transpose = 0;
    /*transpose_step += 1;
    if transpose_step > 128 && transpose_step < 256 {
      transpose = 5;
    } else if transpose_step == 256 {
      transpose_step = 0;
      transpose = 0;
    }*/
    [note + transpose, vel * 0.16] 
  })
