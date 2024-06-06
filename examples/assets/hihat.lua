math.randomseed(0x12345)

return rhythm {
  unit = "1/8",
  pattern = function(context)
    if math.imod(context.pulse_step, 8) == 1 then
      return { 0.8, 0.2, 0.9, 0.2 }
    else
      if math.random() > 0.9 then
        return { 0.8, 0.9 }
      else 
        return { 1 }
      end
    end
  end,
  emit = function(context)
    local note = "c6"
    local _, fraction = math.modf(context.pulse_time_step)
    if fraction == 1.0/2.0 then
      note = "c5 v0.3"
    end
    return note
  end
}