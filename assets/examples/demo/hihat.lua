local pattern = require "pattern"

return emitter {
  unit = "1/16",
  offset = 16 * 8,
  pattern = pattern.euclidean(14, 16, 6),
  emit = function()
    local note_step = 0;
    return function()
      note_step = note_step + 1
      local volume = 1.0 - (note_step % 4) / 4.0
      local key = "c6"
      if note_step % 3 == 0 then
        key = "c5"
        volume = volume * 0.6
      end
      return {key = key, volume = volume}
    end
  end
}
