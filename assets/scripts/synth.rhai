beat_time()
  .every_nth_beat(4)
  .trigger(note_seq([
    [["c 3", 0.5], ["g 3", 0.2]],
    [["off", 0.5], ["off", 0.2]],
    [["c 4", 0.3], ["g 4", 0.2]],
    [["c 3", 0.5], ["g 3", 0.2]],
  ]));