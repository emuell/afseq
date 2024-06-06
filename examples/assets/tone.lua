return rhythm {
  unit = "1/16",
  pattern = pattern.euclidean(6, 8, -5),
  offset = 16 * 64,
  emit = function()
    local SCALE_STEP_COUNT = 4
    local INTERVALS = { 0, 3, 5, 4, 3, 5, 3, 1, 4, 0, 5, 1 }
    local SCALES = {
      scale("c", "mixolydian").notes,
      scale("g", "major").notes,
    }
    ---@param context EmitterContext
    return function(context)
      -- get scale from current step
      local scale_index = math.imod(
        math.floor((context.step - 1) / #INTERVALS / SCALE_STEP_COUNT) + 1,
        #SCALES)
      local notes = pattern.from(INTERVALS):map(function(note_index) 
        return SCALES[scale_index][note_index] or 0 
      end)
      -- get key from current scale and step
      local key
      if context.step % 24 == 1 then
        key = notes[math.random(#INTERVALS)]
      else
        key = notes[math.imod(context.step, #INTERVALS)]
      end
      -- get volume from step
      local volume = (context.step % 3 == 1 or context.step % 7 == 1)
          and 0.25 or math.random(3, 4) / 20;
      volume = volume * 0.5
      -- return final note
      if key == 0 then
        return {}
      else
        return { key = key, volume = volume }
      end
    end
  end
}
