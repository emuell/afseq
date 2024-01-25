---
--- Part of the afseq trait:
--- exports Pattern, a table wrapper with array alike helper functions which are useful to
--- ease creating rhythmical patterns. inspired from https://github.com/rick4stley/array
---
--- Copyright (c) 2024 Eduard Müller <mail@emuell.net>
--- Distributed under the MIT License.
---

----------------------------------------------------------------------------------------------------

---@class pattern
---@operator add(pattern|table): pattern
---@operator mul(number): pattern
local pattern = {}

----------------------------------------------------------------------------------------------------
--- Pattern creation
----------------------------------------------------------------------------------------------------

---Create a new empty pattern.
---@return pattern
function pattern.new()
  local t = {}
  return setmetatable(t, {
    ---all table functions can be accessed as member functions
    __index = pattern,
    ---operator + adds two patterns
    __add = function(a, b)
      return a:copy():add(b)
    end,
    ---operator * creates a repeated pattern
    __mul = function(a, b)
      return a:copy():repeat_n(b)
    end
  })
end

---Create a new pattern from a set of values or tables.
---When passing tables, those will be flattened.
function pattern.from(...)
  return pattern.new():push_back(...)
end

-- create a shallow-copy of the given pattern (or self)
function pattern.copy(self)
  return pattern.from(self)
end

---Create an new pattern which spreads pulses evenly within the given length,
---Using Bresenham’s line algorithm, similar, but not exactly like "euclidean".
---@param steps integer
---@param length integer
---@param offset integer?
---Shortcut for:
---```lua
---pattern.new():init(1, steps):spread(length):rotate(offset)
---```
function pattern.distributed(steps, length, offset)
  assert(type(steps) == "number" and steps > 0, "Invalid step argument")
  assert(type(length) == "number" and length > 0, "Invalid length argument")
  assert(length >= steps, "Length must be >= steps")
  return pattern.new():init(1, steps):spread(length / steps):rotate(offset or 0)
end

---Recursive euclidean pattern impl (implementation detail).
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

---Create a new euclidean rhythm pattern with the given number of pulses in the given
---Length and optionally rotate the contents.
---[Euclidean Rhythm](https://en.wikipedia.org/wiki/Euclidean_rhythm)
---@param pulses integer Number of enabled pulses in the pattern.
---@param length integer Number of total (disabled or enabled) pulses in the pattern.
---@param offset integer? Optional rotation offset.
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
  if offset then
    result:rotate(offset)
  end
  return result
end

----------------------------------------------------------------------------------------------------
--- Access sub ranges
----------------------------------------------------------------------------------------------------

---Shortcut for table.unpack(pattern): returns elements from this pattern as var args.
---@param self table
---@return number[]
function pattern.unpack(self)
  return table.unpack(self)
end

---Get sub range from the pattern as new pattern.
---When the given length is past end of this pattern its filled up with 0s.
---@param i integer
---@param j integer?
function pattern.subrange(self, i, j)
  local len = j or #self
  local a = pattern.new()
  for ii = i, len do
    a:push_back(self[ii] or 0)
  end
  return a
end

---Get first n items from the pattern as new pattern.
---@param length integer
function pattern.take(self, length)
  return self:subrange(1, length)
end

----------------------------------------------------------------------------------------------------
--- Modify contents
----------------------------------------------------------------------------------------------------

---Clear a pattern, remove all its contents
function pattern.clear(self)
  while #self > 0 do
    table.remove(self)
  end
  return self
end

---Fill a pattern with the given value or generator function in length.
---@param value number|fun(index: integer):number
---@param length integer
function pattern.init(self, value, length)
  assert(type(value) ~= "nil", "Invalid value argument")
  assert(type(length) == "number", "Invalid length argument")
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

---Invert the order of items.
function pattern.reverse(self)
  local num = #self
  for i = 1, math.floor(num / 2) do
    local j = num - (i - 1)
    self[j], self[i] = self[i], self[j]
  end
  return self
end

---Shift contents by the given amount to the left (negative amount) or right.
---@param amount integer
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

---Push any number of items or other pattern contents to the end of the pattern.
---When passing tables or patterns, they will be added unpacked.
---@param ... number|table
function pattern.push_back(self, ...)
  local function add_unpacked(v)
    if type(v) == 'table' then
      for i = 1, #v do
        if v[i] ~= nil then
          add_unpacked(v[i])
        end
      end
    else
      table.insert(self, v)
    end
  end
  for i = 1, select("#", ...) do
    local v = select(i, ...)
    add_unpacked(v)
  end
  return self
end

---Alias for pattern.push_back.
pattern.add = pattern.push_back

---Remove an entry from the back of the pattern and returns the popped item.
---@return number
function pattern.pop_back(self)
  return table.remove(self)
end

---Duplicate the pattern n times.
---@param count integer
function pattern.repeat_n(self, count)
  local num = #self
  for _ = 1, count - 1 do
    for i = 1, num do
      self:push_back(self[i])
    end
  end
  return self
end

---Expand (with amount > 1) or shrink (amount < 1) the length of the pattern by the
---given factor, spreading allowed content evenly and filling gaps with 0.
---@param amount number
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
