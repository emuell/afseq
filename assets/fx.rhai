second_time()
  .every_nth_second(8.0)
  .with_offset(64)
  .trigger(note_seq([
    [["C 3", 0.2], ["---"     ], ["---"     ]],
    [["---"     ], ["C 4", 0.2], ["---"     ]],
    [["---"     ], ["---"     ], ["F 4", 0.2]],
]))