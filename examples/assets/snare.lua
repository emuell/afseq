local fun = require "fun"

math.randomseed(0x13ee127)

return rhythm {
  unit = "1/16",
  pattern = fun.cycle { 0, 0, 0, 0, 1, 0, 0.075, 0 }:take(7 * 8):chain { 0, 0, 0, 1, 0, 0, 0.5, 0 }:to_table(),
  emit = function(context)
    -- print(context.step, context.step_value, context.step_time, context.step)
    return { key = "C5", volume = (context.pulse_value == 1) and 0.8 or 0.5 }
  end,
}
