let scale_c = notes_in_scale("c major");
let scale_f = notes_in_scale("f major");
let scale_d = notes_in_scale("d# major");

let samples = [
  sample_pool().load("assets/sequence/MOOGBA4.wav"),
  sample_pool().load("assets/sequence/MOOGBA5.wav"),
  sample_pool().load("assets/sequence/MOOGBA8.wav"),
];

beat_time()
  .every_nth_eighth(1)
  .with_pattern(3, 8)
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