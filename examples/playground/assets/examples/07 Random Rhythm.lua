--[[
  Controlled randomness.
--]]

-- change seeds for other variations...
local PATTERN_SEED = 0x12345
local GATE_SEED    = 0x93752
local EMIT_SEED    = 0x89736

return rhythm {
  unit = "1/8",
  pattern = function(init_context)
    -- randomly emit crammed pulse patterns
    local rand = math.randomstate(PATTERN_SEED)
    return function(context)
      -- every eight pulse
      if math.imod(context.pulse_step, 8) == 1 then
        return { 0.8, 0.2, 0.9, 0.2 }
        -- else random, unlikely
      elseif rand() > 0.9 then
        return { 0.8, 0.9 }
        -- default
      else
        return 1
      end
    end
  end,
  gate = function(init_context)
    -- probability gate
    local rand = math.randomstate(GATE_SEED)
    return function(context)
      return context.pulse_value > rand()
    end
  end,
  emit = function(init_context)
    -- randomly emit a lower or higher note
    local rand = math.randomstate(EMIT_SEED)
    return function(context)
      return rand(1, 10) >= 8 and "c4" or "c5"
    end
  end
}
