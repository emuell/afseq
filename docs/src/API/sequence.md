
<!-- toc -->

# Global {#Global}  

---  
## Functions
### sequence(...[`Note`](../API/note.md#Note) | [`NoteValue`](#NoteValue)) {#sequence}
`->`[`Sequence`](../API/sequence.md#Sequence)  

> Create a sequence from an array of note values or note value varargs.
> 
> Using `sequence` instead of a raw `{}` table can be useful to ease transforming the note
> content and to explicitly pass a sequence of e.g. single notes to the emitter.
> 
> #### examples:
> ```lua
> -- sequence of C4, C5 and an empty note
> sequence(48, "c5", {})
> -- sequence of a +5 transposed C4 and G4 major chord
> sequence("c4'maj", "g4'maj"):transpose(5)
>  ```  



---  
## Aliases  
### NoteValue {#NoteValue}
[`string`](../API/builtins/string.md) | [`number`](../API/builtins/number.md) | [`NoteTable`](../API/note.md#NoteTable) | [`nil`](../API/builtins/nil.md)  
  
  



# Sequence {#Sequence}  

---  
## Properties
### notes : [`NoteTable`](../API/note.md#NoteTable)[][] {#notes}
  

---  
## Functions
### transpose([*self*](../API/builtins/self.md), step : [`integer`](../API/builtins/integer.md) | [`integer`](../API/builtins/integer.md)[]) {#transpose}
`->`[`Sequence`](../API/sequence.md#Sequence)  

> Transpose all notes key values with the specified step value or values.
> 
> Values outside of the valid key range (0 - 127) will be clamped.
> 
> #### examples:
> ```lua
> sequence("c4", "d#5"):transpose(12)
> sequence(note("c'maj"), note("c'maj")):transpose({0, 5})
> ```
### amplify([*self*](../API/builtins/self.md), factor : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#amplify}
`->`[`Sequence`](../API/sequence.md#Sequence)  

> Multiply all notes volume values with the specified factor or factors.
> 
> Values outside of the valid volume range (0 - 1) will be clamped.
> 
> #### examples:
> ```lua
> sequence({"c4 0.5", "g4"}):amplify(0.5)
> sequence("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
> ```
### instrument([*self*](../API/builtins/self.md), instrument : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#instrument}
`->`[`Note`](../API/note.md#Note)  

> Set the instrument attribute of all notes to the specified value or values.
### volume([*self*](../API/builtins/self.md), volume : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#volume}
`->`[`Sequence`](../API/sequence.md#Sequence)  

> Set the volume attribute of all notes to the specified value or values.
> 
> #### examples:
> ```lua
> sequence({"c4", "g4"}):volume(0.5)
> sequence("c'maj"):volume(0.5)
> sequence("c'maj"):volume({0.1, 0.2, 0.3})
> ```
### panning([*self*](../API/builtins/self.md), panning : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#panning}
`->`[`Note`](../API/note.md#Note)  

> Set the panning attribute of all notes to the specified value or values.
### delay([*self*](../API/builtins/self.md), delay : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#delay}
`->`[`Sequence`](../API/sequence.md#Sequence)  

> Set the delay attribute of all notes to the specified value or values.  



