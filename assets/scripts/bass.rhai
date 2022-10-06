let bass_scale = notes_in_scale("c major");

let notes = [bass_scale[0] - 12, "Off", bass_scale[7] - 12, "Off"];

let samples = [
  sample_pool().load("assets/sequence/MOOGBA4.wav"),
  sample_pool().load("assets/sequence/MOOGBA4.wav"),
  sample_pool().load("assets/sequence/MOOGBA1.wav"),
  sample_pool().load("assets/sequence/MOOGBA1.wav"),
];

beat_time()
  .every_nth_sixteenth(2)
  .with_pattern([1,0,1,0,0,0,1,1])
  .with_offset(2)
  .trigger(note_seq([0,1,2,3].map(|i| [samples[i], notes[i], 0.5])));