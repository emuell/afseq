const scales = [
  notes_in_scale("f major"), 
  notes_in_scale("c mixolydian")
];

let note_step = 0;
let scale_index = 0;

let volume_step = 4;
let volume_direction = 1.0;

beat_time()
  .every_sixteenth()
  .with_pattern(euclidian(6, 8).rotate(-5))
  .with_offset(16 * 64)
  .trigger(|| {
    let notes = [0, 5, 2, 3, 7, 2].map(|i| scales[scale_index][i] - 12);
    let note = notes[note_step % notes.len()];
    note_step += 1;
    
    const SCALE_STEP_COUNT = 8;
    scale_index = ((note_step / notes.len()) / SCALE_STEP_COUNT) % scales.len();
    
    const VEL_STEP_COUNT = 32;
    let vel = 0.3 * volume_step / VEL_STEP_COUNT + 0.2;
    if note_step % 2 == 0 {
      vel = 0.3 * (1.0 - volume_step / VEL_STEP_COUNT) + 0.2;
    }
    volume_step += volume_direction;
    if volume_step >= VEL_STEP_COUNT || volume_step == 0 {
      volume_direction = -volume_direction;
    }

    [note, vel * 0.12] 
  })
