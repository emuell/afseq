require "fun"()

return Emitter {
  resolution = 1 / 4,
  pattern = totable(chain(
    take(7 * 8, cycle { 0, 0, 0, 0, 1, 0, 0, 0 }),
    { 0, 0, 0, 1, 0, 0, 1, 0 }
  )),
  emit = { key = "C_4", volume = 1.4 },
}
