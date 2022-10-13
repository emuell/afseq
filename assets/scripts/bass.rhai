let scale_c = notes_in_scale("c major");
let scale_f = notes_in_scale("f major");
let scale_d = notes_in_scale("d# major");

let samples = [
  load_sample("assets/sequence/MOOGBA4.WAV"),
  load_sample("assets/sequence/MOOGBA5.WAV"),
  load_sample("assets/sequence/MOOGBA8.WAV"),
];

beat_time()
  .every_nth_eighth(1)
  .with_pattern(euclidian(3, 8))
  .with_offset(1)
  .trigger(note_seq([
    [samples[0], scale_c[0], 0.5], 
    [], 
    [samples[2], scale_c[0] - 12, 0.5], 
    [samples[0], scale_c[0], 0.5], 
    [samples[2], scale_c[7] - 12, 0.3], 
    [samples[1], scale_c[0] - 12, 0.3]
  ].repeat(4) + [
    [samples[0], scale_f[0], 0.5], 
    [], 
    [samples[2], scale_f[0] - 12, 0.5], 
    [samples[0], scale_f[0], 0.5], 
    [samples[2], scale_f[7] - 12, 0.3], 
    [samples[1], scale_f[0] - 12, 0.3]
  ].repeat(2) + [
    [samples[0], scale_d[0], 0.5], 
    [], 
    [samples[2], scale_d[0] - 12, 0.5], 
    [samples[0], scale_d[0], 0.5], 
    [samples[2], scale_d[7] - 12, 0.3], 
    [samples[1], scale_d[0] - 12, 0.3]
  ].repeat(2)
  ));
