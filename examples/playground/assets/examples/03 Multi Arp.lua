--[[
  Plays an arp from a series of other arps.
--]]

-- Patterns of notes and arpeggio directions
local patterns = {
  -- Change existing or add more patterns as desired
  { notes = { 60, 64, 67 },     direction = "up" },
  { notes = { 64, 67, 71 },     direction = "down" },
  { notes = { 60, 64, 67, 71 }, direction = "up-down" },
  { notes = { 60, 64, 67 },     direction = "down-up" },
}

-- Create Nerdo Rhythm script
return rhythm {
  unit = "1/16",
  emit = function(context)
    -- State variables to track current pattern index and position in pattern
    local pattern_index = 1
    -- Counter to change pattern every 16 steps
    local change_counter = 0
    local change_interval = 16

    return function()
      -- Increment the pattern change counter
      change_counter = change_counter + 1

      -- Change pattern every patternChangeInterval steps
      if change_counter > change_interval then
        pattern_index = pattern_index % #patterns + 1
        change_counter = 0 -- Reset counter
      end

      -- Get the current pattern
      local current_pattern = patterns[pattern_index]
      local notes = current_pattern.notes
      local direction = current_pattern.direction

      -- Calculate the step within the current pattern
      local step = context.step % #notes + 1

      -- Determine the note based on the direction
      local note_index
      if direction == "up" then
        note_index = step
      elseif direction == "down" then
        note_index = #notes - step + 1
      elseif direction == "up-down" then
        note_index = step
        if step > #notes / 2 then
          note_index = #notes - step + 1
        end
      elseif direction == "down-up" then
        note_index = #notes - step + 1
        if step > #notes / 2 then
          note_index = step
        end
      end

      -- Get actual note`
      local note = notes[note_index]
      -- Calculate volume for a dynamic feel (optional)
      local volume = 0.8 + 0.2 * math.cos(context.step / 32 * 2 * math.pi)

      -- Emit the note event
      return {
        key = note,
        volume = volume,
      }
    end
  end
}
