--[[
  Dynamic bouncing ball pulse generator
--]]

return pattern {
  unit = '1/4',
  resolution = 1/32,
  parameter = {
    parameter.integer("distance", 32, { 16, 128 }, "Distance", "Initial bounce distance"),
    parameter.number("speed", 1.025, { 1, 2 }, "Speed", "Bounce speed"),
  },
  pulse = function(init_context) 
    local step, step_size = 0, init_context.parameter.distance
    ---@param context PulseContext
    return function(context)
      if step_size <= 1 then
        -- restart
        step = 0
        step_size = init_context.parameter.distance
        return 0
      end
      if step >= step_size then
        -- modify step with bounce speed
        step_size = step_size / init_context.parameter.speed
        step = 0
      end
      local trigger_new_pulse = (step == 0)
      step = step + 1
      return trigger_new_pulse
    end
  end,
  event = function(context)
    return {
      key = "c4",
      delay = math.random() * 0.25
    }
  end
}