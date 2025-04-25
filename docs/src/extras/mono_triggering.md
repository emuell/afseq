# Rhythm Trigger Modes

When triggering rhythms with notes, e.g. from a MIDI keyboard or in the sequencer of a DAW, a new rhythm instance is started by default for each incoming triggered note.

The rhythm's [`mono`](../API/rhythm.md#mono--boolean) property modifies this behavior.

Monophonic modes can be used to create arpeggio or chord mapping alike rhythms, which consume multiple input notes.

### Polyphonic Mode (`mono = false`) - Default
- Creates a new rhythm instance for each new note.
- Stops each instance when its corresponding note is released.

### Monophonic Mode (`mono = true`)
- Starts the rhythm with the first triggered note.
- Passes all subsequent notes *to the same rhythm* instance.
- Continues running until all notes are released.

## Examples

An arpeggio that cycles through all currently held notes.

```lua
return rhythm {
    mono = true, -- Enable monophonic rhythm triggering
    emit = function(init_context)
        -- Local state for tracking arpeggio position
        local note_index = 0
        return function(context)
            local notes = context.trigger.notes
            if #notes == 0 then 
              -- Skip if no notes held
              return
            end 
            -- Advance and wrap arpeggio position
            note_index = math.imod(note_index + 1, #notes)
            -- Return current note from held chord
            return notes[note_index]
        end
    end
}
```

See [generators](./generators.md) for details of how afseq handles global and local states in general.
