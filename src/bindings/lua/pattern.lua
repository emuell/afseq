---
--- Part of the afseq trait:
--- exports Pattern, a table wrapper with array alike helper functions which are useful to
--- ease creating rhythmical patterns. inspired from https://github.com/rick4stley/array
---
--- Copyright (c) 2023 Eduard Müller <mail@emuell.net>
---

---@diagnostic disable-next-line: lowercase-global
pattern = {}

pattern.mt = {
  -- all table functions can be accessed as member functions
  __index = pattern,
  -- operator + adds two patterns
  __add = function(a, b)
    return a:copy():add(b)
  end,
  -- operator * creates a repeated pattern
  __mul = function(a, b)
    return a:copy():repeat_n(b)
  end
}

----------------------------------------------------------------------------------------------------
--- Pattern creation
----------------------------------------------------------------------------------------------------

-- create a new empty pattern
function pattern.new()
  local a = {}
  setmetatable(a, pattern.mt)
  return a
end

-- create a new pattern from a set of values or tables.
-- when passing tables, those will be flattened.
function pattern.from(...)
  return pattern.new():push_back(...)
end

-- create a shallow-copy of the given pattern (or self)
function pattern.copy(self)
  return pattern.from(self)
end

-- create an new pattern which spreads a pulses evenly within the given length,
-- using Bresenham’s line algorithm, similar, but not exactly like "euclidean".
-- shortcut for pattern.new():init(1, steps):spread(length):rotate(offset)
function pattern.distributed(steps, length, offset)
  assert(type(steps) == "number" and steps > 0, "Invalid step argument")
  assert(type(length) == "number" and length > 0, "Invalid length argument")
  assert(length >= steps, "Length must be >= steps")

  return pattern.new():init(1, steps):spread(length / steps):rotate(offset or 0)
end

-- recursive euclidean pattern impl (implementation detail) 
local function euclidean_impl(front, back)
  local function join_tables(a, b)
    local c = { table.unpack(a) }
    for i = 1, #b do
      table.insert(c, b[i])
    end
    return c
  end
  if #back < 2 then
    return join_tables(front, back)
  end
  local newFront = {}
  while #front > 0 and #back > 0 do
    table.insert(newFront, join_tables(table.remove(front), table.remove(back)))
  end
  return euclidean_impl(newFront, join_tables(front, back))
end

-- create a new euclidean rhythm pattern with the given number of pulses in the given 
-- length and optionally rotate the contents.
-- see https://en.wikipedia.org/wiki/Euclidean_rhythm for details.
function pattern.euclidean(pulses, length, offset)
  assert(type(pulses) == "number" and pulses > 0, "Invalid step argument")
  assert(type(length) == "number" and length > 0, "Invalid length argument")
  assert(type(offset) == "number" or offset == nil, "Invalid offset argument")
  assert(length >= pulses, "length must be >= steps")

  -- initialize
  local front = {}
  for _ = 1, pulses do
    table.insert(front, { 1 })
  end
  local back = {}
  for _ = 1, length - pulses do
    table.insert(back, { 0 })
  end
  -- spread
  local rhythms = euclidean_impl(front, back);
  -- convert to pattern and flatten
  local result = pattern.new();
  for _, g in ipairs(rhythms) do
    result:push_back(g);
  end
  -- rotate
  result:rotate(offset)
  return result
end

----------------------------------------------------------------------------------------------------
--- Access sub ranges
----------------------------------------------------------------------------------------------------

-- shortcut for table.unpack(pattern): returns elements from this pattern as var args.
function pattern.unpack(self)
  return table.unpack(self)
end

-- get sub range from the pattern as new pattern.
-- when the given length is past end of this pattern its filled up with 0s.
function pattern.subrange(self, i, j)
  local len = j or #self
  local a = pattern.new()
  for i = i, len do
    a:push_back(self[i] or 0)
  end 
  return a
end

-- get first n items from the pattern as new pattern
function pattern.take(self, length)
  return self:subrange(1, length)
end

----------------------------------------------------------------------------------------------------
--- Modify contents
----------------------------------------------------------------------------------------------------

-- clear a pattern, remove all its contents
function pattern.clear(self)
  while #self > 0 do
    table.remove(self)
  end
  return self
end

-- fill a pattern with the given value or generator in length
function pattern.init(self, value, length)
  assert(type(value) ~= "nil", "Invalid value argument")
  assert(type(length) == "number" or type(length) == "number", "Invalid length argument")

  self:clear()
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
  for i = 1, math.floor(num / 2) do
    local j = num - (i - 1)
    self[j], self[i] = self[i], self[j]
  end
  return self
end

-- shift contents by the given amount to the left (negative amount) or right
function pattern.rotate(self, amount)
  assert(type(amount) == "number", "Invalid amount parameter")

  local n = #self
  amount = amount % n
  if amount == 0 then return self end
  for i = n, 1, -1 do
    self[i + amount] = self[i]
  end
  for i = 1, amount do
    self[i], self[i + n] = self[i + n], nil
  end
  return self
end

----------------------------------------------------------------------------------------------------
--- Add/remove contents
----------------------------------------------------------------------------------------------------

-- push any number of items or other pattern contents to the end of the pattern
-- when passing tables, patterns, those will be flattened.
function pattern.push_back(self, other, ...)
  local function add_unpacked(v)
    if type(v) == 'table' then
      for i = 1, #v do
        if v[i] ~= nil then
          table.insert(self, v[i])
        end
      end
    else
      table.insert(self, v)
    end
  end
  add_unpacked(other)
  for i = 1, select("#", ...) do
    local v = select(i, ...)
    add_unpacked(v)
  end
  return self
end
-- alias for pattern.push_back
pattern.add = pattern.push_back

-- remove an entry from the back of the pattern
function pattern.pop_back(self)
  return table.remove(self)
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

-- expand (with amount > 1) or shrink (amount < 1) the length of the pattern by the
-- given factor, spreading allowed content evenly and filling gaps with 0
function pattern.spread(self, amount)
  assert(type(amount) == "number" and amount > 0, "Invalid amount parameter")

  local old_num = #self
  local old = self:copy()
  self:init(0, old_num * amount)
  for i = 1, old_num do
    self[math.floor((i - 1) * amount + 0.5) + 1] = old[i]
  end
  return self
end

return pattern
