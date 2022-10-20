const bass_notes = notes_in_scale("c aeolian").map(|n| n);

beat_time()
    .every_nth_eighth(1.0)
    .with_pattern([1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1])
    .trigger(note_seq([
        [bass_notes[0], 0.5],
        [bass_notes[2], 0.5],
        [bass_notes[3], 0.5],
        [bass_notes[0], 0.5],
        [bass_notes[2], 0.5],
        [bass_notes[3], 0.5],
        [bass_notes[6] - 12, 0.5],
    ]))