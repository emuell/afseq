let bass_scale = notes_in_scale("c major");
let notes = [0, 0, 7].map(|i| bass_scale[i] - 12);
let note_step = 0;

beat_time()
  .every_nth_sixteenth(2)
  .with_pattern([1,0,1,0,0,0,1,0])
  .with_offset(2)
  .trigger(note_seq(notes.map(|n| [n, 0.5] )))
