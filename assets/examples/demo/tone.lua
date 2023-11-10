local scales = {
  notes_in_scale("f major"),
  notes_in_scale("c mixolydian"),
}

return Emitter {
  unit = "16th",
  pattern = pattern.euclidean(6, 8, -5),
  offset = 16 * 64,
  emit = function()
    local SCALE_STEP_COUNT = 8
    local VOLUME_STEP_COUNT = 32

    local note_step = 0
    local scale_index = 1

    local volume_step = 4
    local volume_direction = 1.0

    return function()
      -- get current note set
      local notes = fun.to_table(fun.map(function(v)
        return scales[scale_index][v] - 12
      end, { 1, 6, 3, 4, 8, 3 }))
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
      return { key = note, volume = volume * 0.12 }
    end
  end
}
