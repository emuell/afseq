-- local scale = scale("c5", {0,1,3,5,7,9,11}).notes
local scale = scale("c5", "minor").notes

local bass_line = fun.iter({ { 1 }, { 3 }, { 4 }, { 1 }, { 3 }, { 4 }, { 7, -12 } })
    :map(function(x) return scale[x[1]] + (x[2] or 0) end):to_table()

return Emitter {
    unit = "8th",
    pattern = pattern.from({ 1, 0, 1, 0 }, { 0, 1, 0, 0 }, { 1, 0, 1, 0 }, { 0, 1, 0, 1 }),
    emit = sequence { table.unpack(bass_line) }:with_volume(0.7)
}
