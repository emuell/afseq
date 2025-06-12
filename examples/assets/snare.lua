math.randomseed(0x13ee127)

return pattern {
  unit = "1/16",
  pulse = pulse.from { 0, 0, 0, 0, 1, 0, 0.075, 0 } * 7 + { 0, 0, 0, 1, 0, 0, 0.5, 0 },
  gate = function(context)
    return context.pulse_value > math.random()
  end,
  event = function(context)
    return { key = "C5", volume = (context.pulse_value == 1) and 0.8 or 0.5 }
  end,
}
