let scales = [[
    notes_in_scale("c mixolydian"), 
    notes_in_scale("g mixolydian"), 
  ], [
    notes_in_scale("f major7"), 
    notes_in_scale("a# major7"), 
  ]];

let note_step = 0;
let transpose_step = 0;
let scale_index = [0, 0];

let volume_step = 0;
let volume_direction = 1.0;

beat_time()
  .every_sixteenth()
  .with_pattern(7, 16)
  .with_offset(1)
  .trigger(|| {
    let notes = [0, 2, 3, 5, 2, 3, 5].map(|i| scales[scale_index[0]][scale_index[1]][i] - 12);
    let note = notes[note_step % notes.len()];
    note_step += 1;
    scale_index[0] = (note_step / (7*8)) % scales.len();
    scale_index[1] = (note_step / 7) % scales.get(scale_index[0].to_int()).len();
    
    let vel = 0.3 * volume_step / 32.0 + 0.2;
    if note_step % 2 == 0 {
      vel = 0.3 * (1.0 - volume_step / 32) + 0.2;
    }
    volume_step += volume_direction;
    if volume_step >= 32 || volume_step == 0 {
      volume_direction = -volume_direction;
    }
    [note, vel * 0.16] 
  })
