const bass_notes = notes_in_scale("c aeolian");

beat_time()
    .every_eighth()
    .with_pattern([1, 0, 1, 0] + [0, 1, 0, 0] + [1, 0, 1, 0] + [0, 1, 0, 1])
    .trigger(sequence([
        #{key: bass_notes[0], volume: 0.5},
        #{key: bass_notes[2], volume: 0.5},
        #{key: bass_notes[3], volume: 0.5},
        #{key: bass_notes[0], volume: 0.5},
        #{key: bass_notes[2], volume: 0.5},
        #{key: bass_notes[3], volume: 0.5},
        #{key: bass_notes[6] - 12, volume: 0.5},
    ]))