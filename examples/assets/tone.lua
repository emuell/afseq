return pattern {
  unit = "1/16",
  pulse = pulse.euclidean(6, 8, -5),
  repeats = 63,
  offset = 16 * 64,
  event = function(context)
    local SCALE_STEP_COUNT = 8
    local VOLUME_STEP_COUNT = 32

    local SCALES = {
      scale("f", "major").notes,
      scale("c", "mixolydian").notes,
    }

    local scale_index = 1
    local volume_step = 4
    local volume_direction = 1.0

    ---@param context EventContext
    return function(context)
      -- get current note set
      local notes = pulse.from { 1, 6, 3, 4, 8, 3 }:map(function(index, value)
        return SCALES[scale_index][value]
      end)
      -- get current note
      local note = notes[math.imod(context.step, #notes)]
      -- move scale step
      scale_index = (math.floor(math.floor(context.step / #notes) / SCALE_STEP_COUNT) % #SCALES) + 1
      -- move volume step
      local volume = 0.3 * volume_step / VOLUME_STEP_COUNT + 0.2
      if context.step % 2 == 0 then
        volume = 0.3 * (1.0 - volume_step / VOLUME_STEP_COUNT) + 0.2
      end
      volume_step = volume_step + volume_direction
      if volume_step >= VOLUME_STEP_COUNT or volume_step == 0 then
        volume_direction = -volume_direction
      end
      -- return final note
      if note then
        return { key = note, volume = volume * 0.16 }
      else
        return nil
      end
    end
  end
}
