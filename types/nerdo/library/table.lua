---
--- Part of the afseq trait:
--- adds a few extra helper functions to the Lua default tablelib
---

----------------------------------------------------------------------------------------------------

---Create an empty table that uses 'table' functions as methods
---@return table
function table.new()
  return table.create({})
end

---Create or convert a table to an object that uses 'table' functions as methods.
---@param t table?
---@return table
function table.create(t)
  assert(not t or type(t) == 'table')
  return setmetatable(t or {}, {__index = _G.table})
end

---Recursively clears and removes all table elements using the given optional value 
---as new value for all existing items.
---@param cleared unknown? Used internally to keep track of already cleared items.
function table.clear(self, cleared)
  assert(type(self) == 'table')
  cleared = cleared or {}
  for k,v in pairs(self) do
    if (type(v) == 'table' and not cleared[v]) then
      cleared[v] = true
      table.clear(v, cleared)
    end
    self[k] = nil
  end
end

---returns true when the table is empty, else false.
---@return boolean
function table.is_empty(self)
  return self == nil or next(self) == nil
end

---Find element 'value' in table 't', starting from the given index or 1.
---Returns the key that matches the value or nil.
---@param value any
---@param index number?
---@return any?
function table.find(self, value, index)
  local index = index or 1
  local count = 1
  for k, v in pairs(self) do
    if (count >= index and v == value) then
      return k
    end
    count = count + 1
  end
  return nil
end

-- Return a new indexed table with all keys from the given table.
---@return table
function table.keys(self)
  local u = {}
  for k,_ in pairs(self) do
    table.insert(u, k)
  end
  return u
end

---Return a new indexed table with all values from the given table.
---@return table
function table.values(t)
  local u = {}
  for _,v in pairs(t) do
    table.insert(u, v)
  end
  return u
end

---Deeply copy the metatable and all elements of the given table recursively
-- into a new table: create a clone with unique references.
---@return table
function table.rcopy(self)
  assert(type(self) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(self)))
  local lookup_table = {}
  local function _copy(object)
    if (type(object) ~= 'table') then
      return object
    elseif (lookup_table[object] ~= nil) then
      return lookup_table[object]
    else
      local new_table = {}
      lookup_table[object] = new_table
      for k, v in pairs(object) do
        new_table[_copy(k)] = _copy(v)
      end
      return setmetatable(new_table, getmetatable(object))
    end
  end

  return _copy(self)
end

---Copy the metatable and all first level elements of the given table into a
---new table. Use table.rcopy to do a recursive copy of all elements.
---@return table
function table.copy(self)
  assert(type(self) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(self)))
  local new_table = {}
  for k, v in pairs(self) do
    new_table[k] = v
  end
  return setmetatable(new_table, getmetatable(self))
end

---Count the number of items of a table. Also works for non index based tables (using pairs).
---@return number
function table.count(self)
  assert(type(self) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(self)))
  local count = 0
  for _,_ in pairs(self) do
    count = count + 1
  end
  return count
end