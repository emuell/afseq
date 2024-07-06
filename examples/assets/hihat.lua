return rhythm {
  unit = "1/8",
  pattern = function(context)
    local rand = math.randomstate(0x12345)
    return function(context)
      if math.imod(context.pulse_step, 8) == 1 then
        return { 0.8, 0.2, 0.9, 0.2 }
      else
        if rand() > 0.9 then
          return { 0.8, 0.9 }
        else
          return { 1 }
        end
      end
    end
  end,
  emit = function(context)
    local rand = math.randomstate(0x8879)
    return function(context)
      local note = rand(1, 10) >= 8 and "c6" or "c7"
      local _, fraction = math.modf(context.pulse_time_step)
      if fraction == 1.0 / 2.0 then
        note = "c5 v0.3"
      end
      return note
    end
  end
}
