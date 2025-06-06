-- Emit notes with certain probability
return rhythm {
  unit = "1/8",
  pattern = {1, 1, 1, 1},  -- Regular pattern
  emit = function(context)
    if math.random() < 0.3 then  -- 30% chance to emit
      return "c4"
    end
  end
}

-- TRY THIS: Vary probability by step position:
--   if math.random() < (context.step % 4) / 4 then

-- TRY THIS: Higher probability on downbeats: 
--   if math.random() < ((context.pulse_step - 1) % 2 == 0 and 0.8 or 0.2) then
