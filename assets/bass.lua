local notes = notes_in_scale("c aeolian")

local bass_line = fun.to_table(fun.map(function(x)
    if x >= 5 then 
        return notes[x] - 12 
    else 
        return notes[x]
    end
end, { 1, 3, 4, 7, 5, 4, 7, 1, 3, 4, 1, 5, 1, 7 }))

return Emitter {
    unit = "eighth",
    pattern = pattern.euclidean(7, 16, -6),
    emit = sequence(table.unpack(bass_line)):with_volume(0.0)
}