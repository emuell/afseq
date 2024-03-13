local fun = require "fun"

math.randomseed(0x13ee127)

return emitter {
  unit = "1/16",
  pattern = fun.cycle { 0, 0, 0, 0, 1, 0, 0.075, 0 }:take(7 * 8):chain { 0, 0, 0, 1, 0, 0, 0.5, 0 }:to_table(),
  emit = function (context) 
    -- print(context.step_count, context.step_value, context.step_time, context.step_count)
    if context.trigger then
      return { key = "C5", volume = (context.pulse_value == 1) and 1.4 or 0.7} 
    else
      return nil
    end
  end,
}
