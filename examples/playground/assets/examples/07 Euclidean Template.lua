--[[
  Simple euclidean rhythm, with configurable and automateable parameters
--]]

return pattern {
  parameter = {
    parameter.integer('steps', 12, {1, 64}, "Steps", 
      "Number of on steps in the pattern"),
    parameter.integer('pulses', 16, {1, 64}, "Pulses", 
      "Total number of on & off pulses"),
    parameter.integer('offset', 0, {-16, 16}, "Offset", 
      "Rotates on pattern left (values > 0) or right (values < 0)"),
  },
  unit = "1/1",
  pulse = function(context)
    return pulse.euclidean(
      math.min(context.parameter.steps, context.parameter.pulses), 
      context.parameter.pulses, 
      context.parameter.offset)
  end,
  event = "c4"
}