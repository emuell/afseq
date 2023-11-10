globalfun() -- move fun.* into globals

return Emitter {
  unit = "16th",
  -- pattern = to_table(chain(take(7 * 8, cycle { 0, 0, 0, 0, 1, 0, 0, 0 }), { 0, 0, 0, 1, 0, 0, 1, 0 })),
  pattern = pattern.from(0, 1):spread(4) * 7 + { 0, 0, 0, 1 } + { 0, 0, 1, 0 },
  emit = note({ key = "C5" }):with_volume(1.4),
}
