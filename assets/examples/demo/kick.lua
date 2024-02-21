return emitter {
  unit = "1/16",
  pattern = function()
    local values, step = table.create({ 0, 6, 10 }), 0
    return function()
      local pulse = values:find(step % 16) ~= nil
      step = step + 1
      return pulse
    end
  end,
  emit = { 60, 60, note { 60, { key = 96, volume = 0.135 } } },
}
