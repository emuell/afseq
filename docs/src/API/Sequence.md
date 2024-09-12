# Sequence  

<!-- toc -->
  

---  
## Properties
### notes : [`NoteTable`](../API/NoteTable.md)[][] {#notes}
  

---  
## Functions
### amplify([*self*](../API/builtins/self.md), factor : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#amplify}
`->`[`Sequence`](../API/Sequence.md)  

> Multiply all notes volume values with the specified factor or factors.
> 
> Values outside of the valid volume range (0 - 1) will be clamped.
> 
> #### examples:
> ```lua
> sequence({"c4 0.5", "g4"}):amplify(0.5)
> sequence("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
> ```
### delay([*self*](../API/builtins/self.md), delay : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#delay}
`->`[`Sequence`](../API/Sequence.md)  

> Set the delay attribute of all notes to the specified value or values.
### instrument([*self*](../API/builtins/self.md), instrument : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#instrument}
`->`[`Note`](../API/Note.md)  

> Set the instrument attribute of all notes to the specified value or values.
### panning([*self*](../API/builtins/self.md), panning : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#panning}
`->`[`Note`](../API/Note.md)  

> Set the panning attribute of all notes to the specified value or values.
### transpose([*self*](../API/builtins/self.md), step : [`integer`](../API/builtins/integer.md) | [`integer`](../API/builtins/integer.md)[]) {#transpose}
`->`[`Sequence`](../API/Sequence.md)  

> Transpose all notes key values with the specified step value or values.
> 
> Values outside of the valid key range (0 - 127) will be clamped.
> 
> #### examples:
> ```lua
> sequence("c4", "d#5"):transpose(12)
> sequence(note("c'maj"), note("c'maj")):transpose({0, 5})
> ```
### volume([*self*](../API/builtins/self.md), volume : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#volume}
`->`[`Sequence`](../API/Sequence.md)  

> Set the volume attribute of all notes to the specified value or values.
> 
> #### examples:
> ```lua
> sequence({"c4", "g4"}):volume(0.5)
> sequence("c'maj"):volume(0.5)
> sequence("c'maj"):volume({0.1, 0.2, 0.3})
> ```  

