return Emitter {
  unit = "sixteenth",
  offset = 16 * 8,
  pattern = pattern.euclidean(14, 16, 6),
  emit = function()
    local note_step = 0;
    return function()
      note_step = note_step + 1
      local volume = 1.0 - (note_step % 4) / 4.0
      local key = "C 6"
      if note_step % 3 == 0 then
        key = "C 5"
        volume = volume * 0.6
      end
      return note(key):with_volume(volume)
    end
  end
}
