local pattern = require "pattern"

---comment
---@param index integer
---@param size integer
---@return integer
local function iwrap(index, size)
  return (index - 1) % size + 1
end

return emitter {
  unit = "1/4",
  pattern = function(context)
    if iwrap(context.step, 8) == 1 then
      return { 1, 1, 1 }
    else
      if math.random() > 0.9 then
        return { 1, 1 }
      else 
        return { 1 }
      end
    end
  end,
  emit = function(context)
    local key = "c6"
    if iwrap(context.step, context.step_count) == 3 then
      key = "c5"
    end
    return { key = key }
  end
}
