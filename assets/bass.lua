local notes = notes_in_scale("c aeolian")

return Emitter {
    unit = "eighth",
    pattern = { 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1 },
    emit = sequence(notes[1], notes[3], notes[4], notes[1], notes[3], notes[4], notes[7] - 12):amplify(0.5)
}