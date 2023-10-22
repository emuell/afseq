return Emitter {
  resolution = 4 * 4,
  offset = 16 * 4,
  emit = sequence(
    chord("C 3", "D#3", "G 3"):set_volume(0.3),
    chord("C 3", "D#3", "F 3"):set_volume(0.4),
    chord("C 3", "D#3", "G 3"):set_volume(0.3),
    chord("C 3", "D#3", "A#3"):set_volume(0.4)
  ):amplify(1.25)
}
