--[[
  Dynamic bouncing ball pulse generator
--]]

local BOUNCE_DISTANCE = 32
local BOUNCE_SPEED = 1.05

return rhythm {
  unit = '1/4',
  resolution = 1/32,
  pattern = function(context)
    local step, step_size = 0, BOUNCE_DISTANCE
    ---@param context PatternContext
    return function(context)
      if step_size <= 1 then
        -- restart
        step = 0
        step_size = BOUNCE_DISTANCE
        return 0
      end
      if step >= step_size then
        -- modify step with bounce speed
        step_size = step_size / BOUNCE_SPEED
        step = 0
      end
      local trigger_new_pulse = (step == 0)
      step = step + 1
      return trigger_new_pulse
    end
  end,
  emit = function(context)
    return {
      key = "c4",
      delay = math.random() * 0.25
    }
  end
}
