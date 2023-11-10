local scale = notes_in_scale("c aeolian")

local bass_line = fun.to_table(fun.map(function(x)
    if x >= 5 then 
        return scale[x] - 12 
    else 
        return scale[x]
    end
end, { 1, 3, 4, 1, 3, 4, 7 }))

return Emitter {
    unit = "8th",
    pattern = pattern.from({1, 0, 1, 0}, {0, 1, 0, 0}, {1, 0, 1, 0}, {0, 1, 0, 1}),
    emit = sequence(table.unpack(bass_line)):with_volume(0.7)
}