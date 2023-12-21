-- create an empty table that uses 'table' functions as methods
function table.new()
  return table.create({})
end

-- create or convert a table to an object that uses 
-- 'table' functions as methods
function table.create(t)
  assert(not t or type(t) == 'table')
  return setmetatable(t or {}, {__index = _G.table})
end

-- recursively clears and removes all table elements
function table.clear(t, cleared)
  assert(type(t) == 'table')
  cleared = cleared or {}
  for k,v in pairs(t) do
    if (type(v) == 'table' and not cleared[v]) then
      cleared[v] = true
      table.clear(v, cleared)
    end
    t[k] = nil
  end
end

-- returns true when the table is empty, else false
function table.is_empty(t)
  return t == nil or next(t) == nil
end

-- find element 'value' in table 't', starting from index
-- returns the key that matches the value or nil
function table.find(t, value, index)
  local index = index or 1
  local count = 1
  for k, v in pairs(t) do
    if (count >= index and v == value) then
      return k
    end
    count = count + 1
  end
  return nil
end

-- return a list of all keys of a table
function table.keys(t)
  local u = {}
  for k,_ in pairs(t) do
    table.insert(u, k)
  end
  return u
end

-- return a list of all values of a table
function table.values(t)
  local u = {}
  for _,v in pairs(t) do
    table.insert(u, v)
  end
  return u
end

-- deeply copy the metatable and all elements of the given table recursively
-- into a new table - create a clone with unique references.
function table.rcopy(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))

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

  return _copy(t)
end

-- copy the metatable and all first level elements of the given table into a
-- new table. Use table.rcopy to do a recursive copy of all elements
function table.copy(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))
  local new_table = {}
  for k, v in pairs(t) do
    new_table[k] = v
  end
  return setmetatable(new_table, getmetatable(t))
end

-- count the number of items of a table, also works for non index
-- based tables (using pairs).
function table.count(t)
  assert(type(t) == 'table', ("bad argument #1 to 'table.copy' "..
    "(table expected, got '%s')"):format(type(t)))
  local count = 0
  for _,_ in pairs(t) do
    count = count + 1
  end
  return count
end