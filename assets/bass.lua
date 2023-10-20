local notes = notes_in_scale("c aeolian")

local note_emitter = Emitter {
    -- unit = "beats",
    resolution = 1 / 2,
    offset = 8,
    -- duration = nil,
    pattern = { 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1 },
    emit = sequence(
        { key = notes[0], volume = 0.5 },
        { key = notes[2], volume = 0.5 },
        { key = notes[3], volume = 0.5 },
        { key = notes[0], volume = 0.5 },
        { key = notes[2], volume = 0.5 },
        { key = notes[3], volume = 0.5 },
        { key = notes[6] - 12, volume = 0.5 }
    )
}

--[[
    local fx_emitter = Emitter {
        resolution = 1,
        offset = 8,
        trigger = sequence {
            { effects = { "0R23" } },
        }
    }
    
    return note_emitter:join(fx_emitter)
]]

return note_emitter