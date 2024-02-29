local pattern = require "pattern"

return emitter {
  unit = "1/16",
  offset = 16 * 8,
  pattern = pattern.euclidean(14, 16, 6),
  emit = function(initial_context)
    ---@param context EmitterContext
    return function(context)
      local volume = 1.0 - (context.step % 4) / 4.0
      local key = "c6"
      if context.step % 3 == 0 then
        key = "c5"
        volume = volume * 0.6
      end
      return { key = key, volume = volume }
    end
  end
}
