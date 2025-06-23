--[[
  Plays an arp from a series of other arps.
--]]

-- Patterns of notes and arpeggio directions
local patterns = {
  -- Change existing or add more patterns as desired
  { notes = { 48, 52, 55 },     direction = "up" },
  { notes = { 52, 55, 59 },     direction = "down" },
  { notes = { 48, 52, 55, 59 }, direction = "up-down" },
  { notes = { 48, 52, 55 },     direction = "down-up" },
}

return pattern {
  unit = "1/16",
  event = function(context)
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
      local step = math.imod(context.step, #notes)

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

      -- return actual note
      return notes[note_index]
    end
  end
}