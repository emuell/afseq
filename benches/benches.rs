use criterion::criterion_main;

// ---------------------------------------------------------------------------------------------

mod benchmarks;

// ---------------------------------------------------------------------------------------------

#[cfg(any(feature = "scripting", feature = "scripting-no-jit"))]
criterion_main!(
    benchmarks::scripted::scripted, //
    benchmarks::rhythm::rhythm,
    benchmarks::cycle::cycle,
);

#[cfg(not(any(feature = "scripting", feature = "scripting-no-jit")))]
criterion_main!(
    benchmarks::rhythm::rhythm, //
    benchmarks::cycle::cycle,
);
