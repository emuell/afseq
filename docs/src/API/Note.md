# Note  

<!-- toc -->
  

---  
## Properties
### notes : [`NoteTable`](../API/NoteTable.md)[] {#notes}
  

---  
## Functions
### amplify([*self*](../API/builtins/self.md), factor : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#amplify}
`->`[`Note`](../API/Note.md)  

> Multiply the note's volume attribute with the specified factor or factors.
> 
> Values outside of the valid volume range (0 - 1) will be clamped.
> 
> #### examples:
> ```lua
> note({"c4 0.5", "g4"}):amplify(0.5)
> note("c'maj 0.5"):amplify({2.0, 1.0, 0.3})
> ```
### delay([*self*](../API/builtins/self.md), delay : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#delay}
`->`[`Note`](../API/Note.md)  

> Set the note's delay attribute to the specified value or values.
### instrument([*self*](../API/builtins/self.md), instrument : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#instrument}
`->`[`Note`](../API/Note.md)  

> Set the note's instrument attribute to the specified value or values.
### panning([*self*](../API/builtins/self.md), panning : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#panning}
`->`[`Note`](../API/Note.md)  

> Set the note's panning attribute to the specified value or values.
### transpose([*self*](../API/builtins/self.md), step : [`integer`](../API/builtins/integer.md) | [`integer`](../API/builtins/integer.md)[]) {#transpose}
`->`[`Note`](../API/Note.md)  

> Transpose the notes key with the specified step or steps.
> 
> Values outside of the valid key range (0 - 127) will be clamped.
> 
> #### examples:
> ```lua
> note("c4"):transpose(12)
> note("c'maj"):transpose(5)
> note("c'maj"):transpose({0, 0, -12})
> ```
### volume([*self*](../API/builtins/self.md), volume : [`number`](../API/builtins/number.md) | [`number`](../API/builtins/number.md)[]) {#volume}
`->`[`Note`](../API/Note.md)  

> Set the note's volume attribute to the specified value or values.
> 
> #### examples:
> ```lua
> note({"c4", "g4"}):volume(0.5)
> note("c'maj"):volume(0.5)
> note("c'maj"):volume({0.1, 0.2, 0.3})
> ```  

