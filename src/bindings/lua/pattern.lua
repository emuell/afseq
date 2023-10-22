-- table wrapper with array alike helper functions which are useful to create rhythmical patterns
---@diagnostic disable-next-line: lowercase-global
pattern = {}

pattern.mt = {
  __index = pattern,
  __add = function(a, b)
    return a:copy():add(b)
  end
}

-- create a new pattern with the given optional initial value and length
function pattern.new(value, length)
  local a = {}
  setmetatable(a, pattern.mt)
  if value ~= nil and length ~= nil then
    pattern.init(a, value, length)
  end
  return a
end

-- create a new pattern from a set of values or tables
function pattern.from(...)
  local a = pattern.new()
  setmetatable(a, pattern.mt)
  for i = 1, select('#', ...) do
    local v = select(i, ...)
    a:add(v)
  end
  return a
end

-- create a shallow-copy
function pattern.copy(self)
  return pattern.from(self)
end

-- fill a pattern with the given value or generator
function pattern.init(self, value, length)
  if type(value) == 'function' then
    for i = 1, length do
      table.insert(self, value(i))
    end
  else
    for _ = 1, length do
      table.insert(self, value)
    end
  end
  return self
end

-- invert the order of items
function pattern.reverse(self)
  local num = #self
  local swaps = math.floor(num * 0.5)
  for i = 1, swaps do
    local opposite = num - (i - 1)
    local temp = self[opposite]
    self[opposite] = self[i]
    self[i] = temp
  end
  return self
end

-- push any number of items or other pattern contents to the end of the pattern
function pattern.push_back(self, other, ...)
  local values = type(other) == 'table' and { table.unpack(other) } or { other, ... }
  for i = 1, #values do
    if values[i] ~= nil then
      table.insert(self, values[i])
    end
  end
  return self
end

-- append a pattern or set of values (alias of pattern.push_back)
pattern.add = pattern.push_back

-- push any number of items or other pattern contents to the front of the pattern
function pattern.push_front(self, other, ...)
  local values = type(other) == 'table' and { table.unpack(other) } or { other, ... }
  for i = 1, #values do
    if values[i] ~= nil then
      table.insert(self, 1, values[i])
    end
  end
  return self
end

-- duplicate the pattern n times
function pattern.repeat_n(self, count)
  local num = #self
  for _ = 1, count - 1 do
    for i = 1, num do
      self:push_back(self[i])
    end
  end
  return self
end

return pattern
