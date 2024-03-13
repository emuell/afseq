local pattern = require "pattern"
local fun = require "fun"

local scales = {
  scale("f", "major").notes,
  scale("c", "mixolydian").notes,
}

return emitter {
  unit = "1/16",
  pattern = pattern.euclidean(6, 8, -5),
  offset = 16 * 64,
  emit = function()
    local SCALE_STEP_COUNT = 8
    local VOLUME_STEP_COUNT = 32

    local note_step = 2
    local scale_index = 1

    local volume_step = 4
    local volume_direction = 1.0

    return function()
      -- get current note set
      local notes = fun.iter({ 1, 6, 3, 4, 0, 3 })
          :map(function(note_index) return scales[scale_index][note_index] or 0 end)
          :totable()
      -- move note step
      local note = notes[math.floor(note_step % #notes) + 1]
      note_step = note_step + 1
      -- move scale step
      scale_index = (math.floor(math.floor(note_step / #notes) / SCALE_STEP_COUNT) % #scales) + 1
      -- move volume step
      local volume = 0.3 * volume_step / VOLUME_STEP_COUNT + 0.2
      if note_step % 2 == 0 then
        volume = 0.3 * (1.0 - volume_step / VOLUME_STEP_COUNT) + 0.2
      end
      volume_step = volume_step + volume_direction
      if volume_step >= VOLUME_STEP_COUNT or volume_step == 0 then
        volume_direction = -volume_direction
      end
      -- return final note
      if note == 0 then
        return {}
      else
        return { key = note, volume = volume * 0.25 }
      end
    end
  end
}
