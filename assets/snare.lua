globalfun() -- move fun.* into globals

return Emitter {
  resolution = 1 / 4,
  pattern_fun = totable(chain(
    take(7 * 8, cycle { 0, 0, 0, 0, 1, 0, 0, 0 }),
    { 0, 0, 0, 1, 0, 0, 1, 0 }
  )),
  pattern = pattern.from({ 0, 0, 0, 0 }, { 1, 0, 0, 0 }):repeat_n(7) +
      pattern.from({ 0, 0, 0, 1 }, { 0, 0, 1, 0 }),
  emit = { key = "C_4", volume = 1.4 },
}
