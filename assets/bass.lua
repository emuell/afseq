local notes = notes_in_scale("c aeolian")

local note_emitter = Emitter {
    -- unit = "beats",
    resolution = 1 / 2,
    offset = 8 * 4,
    -- duration = nil,
    pattern = { 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1 },
    emit = sequence(notes[1], notes[3], notes[4], notes[1], notes[3], notes[4], notes[7] - 12):amplify(0.5)
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
