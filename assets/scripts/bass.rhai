let bass_notes = notes_in_scale("c mixolydian");

beat_time()
  .every_nth_eighth(1)
  .with_pattern([0,1,0,1, 1,0,0,1] + [0,1,0,1, 0,0,0,1])
  .trigger(note_seq([0, 0, 2, 1, 0, 0, 1].map(|v| { [bass_notes[v], 0.2] } )));
